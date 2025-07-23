use pest::Parser;
use pest_derive::Parser;
use crate::error::Result;

use std::{fs, path::Path};

#[derive(Parser)]
#[grammar = "src/schema/schema.pest"]
pub struct SchemaParser;

