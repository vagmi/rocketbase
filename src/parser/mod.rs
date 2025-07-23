use pest::Parser;
use std::{fs, path::Path};
use crate::error::Result;
use crate::schema::{SchemaParser, Rule};

pub fn parse_schema(path: &Path) -> Result<()> {
    let unparsed_file = fs::read_to_string(path)?;
    let schema = SchemaParser::parse(Rule::schema, &unparsed_file)?.next().unwrap();
    println!("Parsed schema: {:#?}", schema);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn test_parse_schema() {
        let path = PathBuf::from("tests/schemas/blog.schema");
        let res = parse_schema(&path);
        println!("Result: {:?}", res);
        assert!(res.is_ok());
    }
}
