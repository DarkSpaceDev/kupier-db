use crate::parser::KlangParser;
use crate::parser::Rule;
use pest::Parser;

// #[test]
// fn null() {
//     parses_ok(Rule::null, "NULL");
// }

// #[test]
// fn and_ampersand() {
//     parses_ok(Rule::and, "&&");
// }

// #[test]
// fn and() {
//     parses_ok(Rule::and, "and");
// }

// #[test]
// fn conditional_stmt_two_variables() {
//     parses_ok(Rule::conditional_stmt, "blarg = yarg");
// }

// #[test]
// fn conditional_stmt_variable_eq_scalar() {
//     parses_ok(Rule::conditional_stmt, "blarg = 7");
// }

// #[test]
// fn conditional_stmt_scalar_eq_variable() {
//     parses_ok(Rule::conditional_stmt, "7 = yarg.garg");
// }

#[test]
fn conditional_stmt_binary_term_scalar_int_ok() {
    parses_ok(Rule::BinaryTerm, "1");
}

#[test]
fn conditional_stmt_binary_term_identifier_ok() {
    parses_ok(Rule::BinaryTerm, "xyz");
}

#[test]
fn conditional_stmt_scalar_eq_scalar() {
    parses_ok(Rule::BinaryExpr, "0 = 1");
}

#[test]
fn number_unsigned_int() {
    parses_ok(Rule::Int, "7");
}

#[test]
fn number_signed_int() {
    parses_ok(Rule::Int, "-7000");
}

// #[test]
// fn scalar_date_ok() {
//     parses_ok(Rule::scalar, "2023-04-05");
// }

// #[test]
// fn scalar_weird_value() {
//     parses_not_ok(Rule::Int, "550-551-552");
// }

// fn compare<T: PartialEq + std::fmt::Debug>(expected: Vec<T>, actual: Vec<T>) {
//     //println!("------------------------------");
//     //println!("tokens   = {:?}", actual);
//     //println!("expected = {:?}", expected);
//     //println!("------------------------------");
//     assert_eq!(expected, actual);
// }

#[allow(dead_code)]
fn parses_not_ok(rule: Rule, input: &str) {
    let result = KlangParser::parse(rule, input);

    if !result.is_err() {
        println!("{:?}", result.clone().unwrap().as_str())
    }

    assert!(result.is_err());
}

fn parses_ok(rule: Rule, input: &str) {
    //println!("------------------------------");
    //println!("tokens   = {:?}", actual);
    //println!("expected = {:?}", expected);
    //println!("------------------------------");

    let result = KlangParser::parse(rule, input);

    if result.is_err() {
        println!("{:?}", result.clone().err());
    }

    assert!(result.is_ok());

    assert_eq!(result.unwrap().as_str(), input.trim());
}
