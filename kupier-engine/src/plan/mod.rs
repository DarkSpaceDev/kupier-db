use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::types::expression::Expression;

/// https://github.com/erikgrinaker/toydb/blob/master/src/sql/plan/mod.rs

/// A query plan
#[derive(Debug)]
pub struct QueryPlan(pub Node);

impl Display for QueryPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl QueryPlan {
    pub fn from_ast(query_expr: &kupier_lang::ast::QueryExpr) -> QueryPlan {
        let collection_scan = Node::CollectionScan(CollectionScan {
            alias: query_expr.table.alias.clone(),
            collection: query_expr.table.value.clone(),
            expr: Option::None,
            schema: String::from_str("default").unwrap(),
        });

        return QueryPlan(collection_scan);
    }
}

/// A plan node
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    CollectionScan(CollectionScan),
}

/// A query plan
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectionScan {
    // pub source: Box<Node>,
    pub schema: String,
    pub collection: String,
    pub alias: Option<String>,
    pub expr: Option<Expression>,
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "{}", self.format("".into(), true, true))
        write!(f, ">> todo >>")
    }
}
