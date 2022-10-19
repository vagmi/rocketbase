use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::column_type::{ColumnType, RelationType};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub id: Uuid,
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

pub trait FindById<T> {
    fn find_def(&self, id: Uuid) -> Option<&T>;
}

impl FindById<ColumnDef> for Vec<ColumnDef> {
    fn find_def(&self, id: Uuid) -> Option<&ColumnDef> {
        self.iter().find(|cd| { cd.id == id })
    }
}