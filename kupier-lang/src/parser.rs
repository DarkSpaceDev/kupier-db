use pest::{iterators::Pairs, Parser};

use crate::ast::{BinaryExpr, BinaryOp, IdentityValue, Node, QueryExpr, ScalarValue};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct KlangParser;

pub fn parse_query(source: &str) -> std::result::Result<Vec<Node>, pest::error::Error<Rule>> {
    let mut ast = vec![];
    let pairs = KlangParser::parse(Rule::Statement, source)?;

    for pair in pairs {
        match pair.as_rule() {
            Rule::Query => {
                ast.push(build_ast_from_query_expr(pair));
            }
            Rule::EOI => {
                break;
            }
            _ => {
                todo!("Undefined Rule: {:?}", pair.as_rule());
            }
        }
    }

    Ok(ast)
}

fn build_ast_from_query_expr(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Query => {
            let mut inner = pair.into_inner();
            let table = parse_identity(inner.next().unwrap());

            let mut query_expr = QueryExpr {
                table: table.try_into().unwrap(),
                filter: Vec::new(),
                order: Vec::new(),
            };

            while inner.len() > 0 {
                let inner_pair = inner.next().unwrap();
                let rule = inner_pair.as_rule();

                match rule {
                    Rule::WhereClause => {
                        query_expr
                            .filter
                            .push(parse_binary_expr(inner_pair.into_inner().next().unwrap()));
                    }
                    invalid => panic!("Invalid Rule! {:?}", invalid),
                }
            }

            println!("Parsed ...");

            return Node::Query(query_expr);
        }
        unknown => panic!("Unknown expression: {:?}", unknown),
    }
}

fn parse_binary_expr(pair: pest::iterators::Pair<Rule>) -> BinaryExpr {
    match pair.as_rule() {
        Rule::BinaryExpr => {
            let mut inner_rules = pair.into_inner();
            let lhs = parse_binary_term(inner_rules.next().unwrap());
            let op = parse_binary_op(inner_rules.next().unwrap());
            let rhs = parse_binary_term(inner_rules.next().unwrap());

            let mut lhs_is_binaryexpr = false;
            let mut rhs_is_binaryexpr = false;

            match &lhs {
                Node::BinaryExpr(_) => {
                    lhs_is_binaryexpr = true;
                }
                _ => (),
            }

            match &rhs {
                Node::BinaryExpr(_) => {
                    rhs_is_binaryexpr = true;
                }
                _ => (),
            }

            if (op == BinaryOp::And || op == BinaryOp::Or)
                && (!rhs_is_binaryexpr || !lhs_is_binaryexpr)
            {
                panic!("Invalid use of OR|AND operator. The left and right expressions must be boolan evaluations.")
            }

            return BinaryExpr {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            };
        }
        unknown => panic!("Unknown expression: {:?}", unknown),
    }
}

fn parse_binary_op(pair: pest::iterators::Pair<Rule>) -> BinaryOp {
    match pair.as_rule() {
        Rule::BinaryOp => parse_binary_op(pair.into_inner().next().unwrap()),
        Rule::And => BinaryOp::And,
        Rule::Or => BinaryOp::Or,
        Rule::Eq => BinaryOp::Eq,
        Rule::Ne => BinaryOp::Ne,
        Rule::Gt => BinaryOp::Gt,
        Rule::GtEq => BinaryOp::GtEq,
        Rule::Lt => BinaryOp::Lt,
        Rule::LtEq => BinaryOp::LtEq,
        unknown => panic!("Unknown operator: {:?}", unknown),
    }
}

fn parse_binary_term(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::BinaryExpr => {
            return Node::BinaryExpr(parse_binary_expr(pair));
        }
        Rule::ScalarValue => {
            return parse_scalar_value(pair);
        }
        Rule::IdentifierPath => {
            return parse_identity(pair);
        }
        Rule::BinaryTerm => {
            return parse_binary_term(pair.into_inner().next().unwrap());
        }
        unknown => panic!("Unknown expression: {:?}", unknown),
    }
}

fn parse_identity(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::IdentifierStmt => {
            let mut pair = pair.into_inner();
            let identity = pair.next().unwrap();
            let alias = pair.next();

            if alias.is_none() {
                return Node::Identity(IdentityValue {
                    value: identity.as_str().to_owned(),
                    alias: Option::None,
                });
            }

            Node::Identity(IdentityValue {
                value: identity.as_str().to_owned(),
                alias: Some(
                    alias
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .to_owned(),
                ),
            })
        }
        Rule::IdentifierPath => Node::Identity(IdentityValue {
            value: pair.as_str().to_owned(),
            alias: Option::None,
        }),
        unknown => panic!("Unknown identity: {:?}", unknown),
    }
}

fn parse_scalar_value(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::ScalarValue => {
            return parse_scalar_value(pair.into_inner().next().unwrap());
        }
        Rule::Int => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "-" => (-1, &istr[1..]),
                _ => (1, istr),
            };
            let int: i64 = istr.parse().unwrap();
            Node::Scalar(ScalarValue::Int(sign * int))
        }
        Rule::Decimal => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "-" => (-1.0, &istr[1..]),
                _ => (1.0, istr),
            };
            let float: f64 = istr.parse().unwrap();
            Node::Scalar(ScalarValue::Decimal(sign * float))
        }
        Rule::String => {
            let str = pair.as_str();
            Node::Scalar(ScalarValue::String(String::from(str)))
        }
        Rule::Boolean => {
            let str = pair.as_str();
            let value = str.parse().unwrap();
            Node::Scalar(ScalarValue::Boolean(value))
        }
        Rule::Null => Node::Scalar(ScalarValue::Null),
        Rule::Undefined => Node::Scalar(ScalarValue::Undefined),
        unknown => panic!("Unknown scalar value: {:?}", unknown),
    }
}

#[cfg(test)]
pub mod tests {
    use crate::parser::KlangParser;
    use crate::parser::Rule;
    use pest::Parser;

    use super::*;

    fn get_inner_pair(mut iterator: Pairs<Rule>) -> Pairs<Rule> {
        assert_eq!(iterator.len(), 1);

        iterator.next().unwrap().into_inner()
    }

    #[test]
    fn query_ok() {
        let _ = KlangParser::parse(Rule::Query, "abcd").unwrap();
    }

    #[test]
    fn identifier_simple_ok() {
        let result = KlangParser::parse(Rule::IdentifierStmt, "abcd").unwrap();
        for pair in result {
            let node = parse_identity(pair);
            let some_value = node.try_into() as Result<IdentityValue, ()>;
            assert!(some_value.is_ok());

            let value = some_value.unwrap();
            assert_eq!(value.value, "abcd");
            assert_eq!(value.alias, Option::None);
        }
    }

    #[test]
    fn identifier_ok() {
        let result = KlangParser::parse(Rule::IdentifierStmt, "abcd.efgh").unwrap();
        for pair in result {
            let node = parse_identity(pair);
            let some_value = node.try_into() as Result<IdentityValue, ()>;
            assert!(some_value.is_ok());

            let value = some_value.unwrap();
            assert_eq!(value.value, "abcd.efgh");
            assert_eq!(value.alias, Option::None);
        }
    }

    #[test]
    fn identifier_with_alias_ok() {
        let result = KlangParser::parse(Rule::IdentifierStmt, "abcd.efgh AS woah").unwrap();

        for pair in result {
            let node = parse_identity(pair);
            let some_value = node.try_into() as Result<IdentityValue, ()>;
            assert!(some_value.is_ok());

            let value = some_value.unwrap();
            assert_eq!(value.value, "abcd.efgh");
            assert_eq!(value.alias, Some(String::from("woah")));
        }
    }

    #[test]
    fn scalar_value_int_ok() {
        let result = KlangParser::parse(Rule::ScalarValue, "7").unwrap();
        let mut inner_pair = get_inner_pair(result);

        let scalar_value = inner_pair.next().unwrap();
        let node = parse_scalar_value(scalar_value);

        let value = node.try_into() as Result<ScalarValue, ()>;
        assert!(value.is_ok());

        // tuple struct
        let scalar = value.unwrap().try_into() as Result<i64, ()>;
        assert!(scalar.is_ok());
        assert_eq!(scalar.unwrap(), 7);
    }

    #[test]
    fn atomic_where_clause_ok() {
        let _ = KlangParser::parse(Rule::AtomicClause, "| where x = y").unwrap();
    }

    #[test]
    fn atomic_where_clause2_ok() {
        let _ = KlangParser::parse(Rule::AtomicClause, "| where x = y and b = c").unwrap();
    }

    #[test]
    fn where_clause_ok() {
        let _ = KlangParser::parse(Rule::WhereClause, "WHERE x = y").unwrap();
    }

    #[test]
    fn binary_expression_ok() {
        let _ = KlangParser::parse(Rule::BinaryExpr, "x = y").unwrap();
    }

    #[test]
    fn binary_expression_and_ok() {
        let _ = KlangParser::parse(Rule::BinaryExpr, "x = y and x = 3").unwrap();
    }

    #[test]
    fn binary_expression_and_grouped_ok() {
        let _ = KlangParser::parse(Rule::BinaryExpr, "x = y and (x = y)").unwrap();
    }

    #[test]
    fn binary_expression_or_grouped_ok() {
        let _ = KlangParser::parse(Rule::BinaryExpr, "x = y or (x = y)").unwrap();
    }

    #[test]
    fn binary_expression_double_grouped_or_ok() {
        let result = KlangParser::parse(Rule::BinaryExpr, "(x = y) or (x = y)");

        if result.is_err() {
            println!("Error: {:?}", result.clone().err());
        }

        assert!(result.is_ok());
    }

    #[test]
    fn binary_expression_complex_ok() {
        let result = KlangParser::parse(
            Rule::BinaryExpr,
            "(x = y or x = y) and (b = c or (d = f and g = h))",
        );

        if result.is_err() {
            println!("Error: {:?}", result.clone().err());
        }

        assert!(result.is_ok());
    }
}
