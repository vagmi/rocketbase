use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

pub mod column_def;
pub mod column_type;

pub use self::column_def::{ColumnDef, FindById};
pub use self::column_type::ColumnType;

#[derive(Debug)]
pub enum ColumnChange<'a> {
    AddColumn(&'a ColumnDef),
    RenameColumn(String, String),
    ChangeType(String, ColumnType),
}

impl<'a> ColumnChange<'a> {
    pub fn get_alter_statement(&'a self) -> String {
        match self {
            Self::AddColumn(cd) => format!(
                "add column {} {} {} {}",
                cd.name,
                cd.column_type.pg_sql_type(),
                cd.is_unique(),
                cd.is_not_null()
            ),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CollectionError {
    #[error("SQLX Error")]
    SqlxError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection {
    pub name: String,
    pub column_defs: Vec<ColumnDef>,
}

impl Collection {
    pub fn create_table_statement(&self) -> String {
        let mut stmt = String::new();
        let cds = self
            .column_defs
            .iter()
            .map(|cd| cd.sql_column_def_for_insert())
            .collect::<Vec<String>>()
            .join(",");
        stmt.push_str(
            format!(
                "create table if not exists {}( \
			id bigserial primary key, \
			created_at timestamptz not null default now(), \
			updated_at timestamptz, \
			{} \
		)",
                self.name, cds
            )
            .as_str(),
        );
        stmt
    }

    pub async fn update_collection<'a, E>(
        &self,
        ex: E,
        other: &Collection,
    ) -> Result<(), CollectionError>
    where
        E: 'a + Executor<'a, Database = Postgres>,
    {
        let mut changes: Vec<ColumnChange> = vec![];
        other.column_defs.iter().for_each(|cd| {
            match self.column_defs.find_def(cd.id) {
                Some(orig_cd) => {
                    if orig_cd == cd { return; }; 
                    unimplemented!()
                }
                None => {
                    changes.push(ColumnChange::AddColumn(cd));
                }
            };
        });
        let column_change_statements = changes
            .iter()
            .map(|ch| ch.get_alter_statement())
            .collect::<Vec<String>>()
            .join(",");
        let mut alter_stmt = String::new();
        alter_stmt
            .push_str(format!("alter table {} {}", self.name, column_change_statements).as_str());
        ex.execute(alter_stmt.as_str())
            .await
            .map_err(|_| CollectionError::SqlxError)?;
        Ok(())
    }

    pub async fn create_collection<'a, E>(&self, ex: E) -> Result<(), CollectionError>
    where
        E: 'a + Executor<'a, Database = Postgres>,
    {
        let create_stmt = self.create_table_statement();
        println!("{}", create_stmt);
        ex.execute(create_stmt.as_str())
            .await
            .map_err(|_| CollectionError::SqlxError)?;
        Ok(())
    }
    // pub fn create_column<'a, E>(&mut self, ex: E) -> Result<(), CollectionError>
    // where E: 'a + Executor<Database = Postgres>
    // {
    // 	Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    #[derive(sqlx::FromRow)]
    #[allow(dead_code)]
    struct Organization {
        id: i64,
        name: String,
        created_at: chrono::DateTime<Utc>,
        updated_at: Option<chrono::DateTime<Utc>>,
    }
    #[derive(sqlx::FromRow)]
    #[allow(dead_code)]
    struct UpdatedOrganization {
        id: i64,
        name: String,
        website: String,
        created_at: chrono::DateTime<Utc>,
        updated_at: Option<chrono::DateTime<Utc>>,
    }
    use super::*;

    #[test]
    fn should_create_create_stmt() {
        let expected_stmt = r#"create table if not exists users( id bigserial primary key, created_at timestamptz not null default now(), updated_at timestamptz, name text unique not null )"#;
        let coll = Collection {
            name: "users".into(),
            column_defs: vec![ColumnDef {
                id: Uuid::new_v4(),
                name: "name".into(),
                column_type: ColumnType::Text,
                required: true,
                unique: true,
            }],
        };
        let ct_stmt = coll.create_table_statement();
        assert_eq!(expected_stmt, ct_stmt, "create statement match failed");
    }
    #[sqlx::test]
    async fn should_update_collection(pool: sqlx::PgPool) {
        let mut conn = pool.acquire().await.expect("unable to get a connection");
        let name_id = Uuid::new_v4();
        let coll = Collection {
            name: "organizations".into(),
            column_defs: vec![ColumnDef {
                id: name_id,
                name: "name".into(),
                column_type: ColumnType::Text,
                required: true,
                unique: true,
            }],
        };
        let new_def = Collection {
            name: "organizations".into(),
            column_defs: vec![
                ColumnDef {
                    id: name_id,
                    name: "name".into(),
                    column_type: ColumnType::Text,
                    required: true,
                    unique: true,
                },
                ColumnDef {
                    id: Uuid::new_v4(),
                    name: "website".into(),
                    column_type: ColumnType::Text,
                    required: true,
                    unique: true,
                },
            ],
        };
        coll.create_collection(&mut conn)
            .await
            .expect("unable to create collection");
        coll.update_collection(&mut conn, &new_def)
            .await
            .expect("Could not update collection");
        let res = conn
            .execute("insert into organizations(name, website) values('tarkalabs', 'tarkalabs.com')")
            .await
            .expect("unable to insert org");
        assert_eq!(
            1,
            res.rows_affected(),
            "expected one row to be affected. affected {} rows",
            res.rows_affected()
        );
        let orgs = sqlx::query_as::<_, UpdatedOrganization>("select * from organizations")
            .fetch_all(&mut conn)
            .await
            .expect("unable to query orgs");

        assert_eq!(orgs.len(), 1, "expected 1 row found {} rows", orgs.len());
    }
    #[sqlx::test]
    async fn should_create_collection(pool: sqlx::PgPool) {
        let mut conn = pool.acquire().await.expect("unable to get a connection");
        let coll = Collection {
            name: "organizations".into(),
            column_defs: vec![ColumnDef {
                id: Uuid::new_v4(),
                name: "name".into(),
                column_type: ColumnType::Text,
                required: true,
                unique: true,
            }],
        };
        coll.create_collection(&mut conn)
            .await
            .expect("unable to create collection");
        let res = conn
            .execute("insert into organizations(name) values('tarkalabs')")
            .await
            .expect("unable to insert org");
        assert_eq!(
            1,
            res.rows_affected(),
            "expected one row to be affected. affected {} rows",
            res.rows_affected()
        );
        let orgs = sqlx::query_as::<_, Organization>("select * from organizations")
            .fetch_all(&mut conn)
            .await
            .expect("unable to query orgs");

        assert_eq!(orgs.len(), 1, "expected 1 row found {} rows", orgs.len());
    }
}
