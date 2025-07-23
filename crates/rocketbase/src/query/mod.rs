use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/query/query.pest"]
pub struct QueryParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_parse_query() {
        let query = r#"{
            query {
                # filter by id (notice the singular)
                user(id: "uuid-goes-here") {
                    id
                    email
                    posts(first: 10)  {
                        edges {
                            cursor
                        }
                       nodes {
                        id
                        content
                        published_at
                        comments_count # notice the relation (should resolve to a number)
                       }
                       pageInfo {
                        count
                        hasNext
                       }
                    }
                }
            }
        }"#;
        let res = QueryParser::parse(Rule::query, query);
        println!("Parsed query: {:#?}", res);
        assert!(res.is_ok());
    }
}
