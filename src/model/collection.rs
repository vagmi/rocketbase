use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};

#[derive(Debug, thiserror::Error)]
pub enum CollectionError {
    #[error("SQLX Error")]
    SqlxError,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RelationType {
    ManyToOne(String),
    OneToOne(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ColumnType {
    UUID,
    Int,
    Decimal,
    Text,
    JSON,
    Email,
    Url,
    User,
    Relation(RelationType),
}
impl ColumnType {
    pub fn pg_sql_type(&self) -> String {
        match  &self {
            Self::UUID => "uuid".into(),
            Self::Int => "i64".into(),
            Self::Decimal => "decimal".into(),
            Self::Text => "text".into(),
            Self::JSON => "jsonb".into(),
            Self::Email => "text".into(),
            Self::Url => "text".into(),
            Self::User => "bigint".into(),
            Self::Relation(RelationType::ManyToOne(_)) => "bigint".into(),
            _ => unimplemented!("unknown type"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    pub required: bool,
    pub unique: bool,
}

impl ColumnDef {
    pub fn sql_column_def_for_insert(&self) -> String {
        let mut buf: String = String::new();
        match &self.column_type {
            ColumnType::User => buf.push_str(
                format!(
                    "{} bigint {} {} references users(id)",
                    self.name,
                    self.is_unique(),
                    self.is_not_null()
                )
                .as_str(),
            ),
            ColumnType::Relation(RelationType::ManyToOne(table)) => buf.push_str(
                format!(
                    "{} bigint {} references {}(id)",
                    self.name,
                    self.is_not_null(),
                    table
                )
                .as_str(),
            ),
            ColumnType::Relation(RelationType::OneToOne(table)) => buf.push_str(
                format!(
                    "{} bigint unique {} references {}(id)",
                    self.name,
                    self.is_not_null(),
                    table
                )
                .as_str(),
            ),
            _ => buf.push_str(
                format!(
                    "{} {} {} {}",
                    self.name,
                    self.column_type.pg_sql_type(),
                    self.is_unique(),
                    self.is_not_null()
                )
                .as_str(),
            ),
        }
        buf
    }
    pub fn is_unique(&self) -> String {
        match self.unique {
            true => "unique".into(),
            _ => "".into(),
        }
    }
    pub fn is_not_null(&self) -> String {
        match self.required {
            true => "not null".into(),
            _ => "".into(),
        }
    }
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

    #[derive(sqlx::FromRow)]
    #[allow(dead_code)]
    struct Organization {
        id: i64,
        name: String,
        created_at: chrono::DateTime<Utc>,
        updated_at: Option<chrono::DateTime<Utc>>,
    }
    use super::*;

    #[test]
    fn should_create_create_stmt() {
			let expected_stmt=r#"create table if not exists users( id bigserial primary key, created_at timestamptz not null default now(), updated_at timestamptz, name text unique not null )"#;
        let coll = Collection {
            name: "users".into(),
            column_defs: vec![ColumnDef {
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
    async fn should_create_collection(pool: sqlx::PgPool) {
        let mut conn = pool.acquire().await.expect("unable to get a connection");
        let coll = Collection {
            name: "organizations".into(),
            column_defs: vec![ColumnDef {
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
