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
            Self::RenameColumn(old_name, new_name) => format!(
                "rename column {} to {}", old_name, new_name
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
                    if orig_cd.name != cd.name {
                        changes.push(ColumnChange::RenameColumn(orig_cd.name.clone(), cd.name.clone()));
                        return; 
                    }
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
mod tests;