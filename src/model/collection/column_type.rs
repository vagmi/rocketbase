use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RelationType {
    ManyToOne(String),
    OneToOne(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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