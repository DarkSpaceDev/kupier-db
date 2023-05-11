/*
   NOTES: Missing Copy and Eq
*/

/// Represents a binary operation for comparison or composition.
#[derive(Clone, PartialEq, Debug)]
pub enum BinaryOp {
    Eq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Ne,
    And,
    Or,
}

#[derive(Clone, PartialEq, Debug)]
pub enum OrderByDirection {
    Asc,
    Desc,
}

#[derive(Clone, PartialEq, Debug)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Node>,
    pub right: Box<Node>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ScalarValue {
    Int(i64),
    Decimal(f64),
    String(String),
    Boolean(bool),
    Date(String),
    Null,
    Undefined,
}

#[derive(Clone, PartialEq, Debug)]
pub struct IdentityValue {
    pub value: String,
    pub alias: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct OrderByClause {
    pub identity: IdentityValue,
    pub direction: OrderByDirection,
}

#[derive(Clone, PartialEq, Debug)]
pub struct QueryExpr {
    pub table: IdentityValue,
    pub filter: Vec<BinaryExpr>,
    pub order: Vec<OrderByClause>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Node {
    Identity(IdentityValue),
    Scalar(ScalarValue),
    BinaryExpr(BinaryExpr),
    Query(QueryExpr),
}

impl TryInto<BinaryExpr> for Node {
    type Error = ();

    fn try_into(self) -> Result<BinaryExpr, Self::Error> {
        if let Node::BinaryExpr(value) = &self {
            return Ok(value.clone());
        }

        Err(())
    }
}

impl TryInto<IdentityValue> for Node {
    type Error = ();

    fn try_into(self) -> Result<IdentityValue, Self::Error> {
        if let Node::Identity(value) = &self {
            return Ok(value.clone());
        }

        Err(())
    }
}

impl TryInto<ScalarValue> for Node {
    type Error = ();

    fn try_into(self) -> Result<ScalarValue, Self::Error> {
        if let Node::Scalar(value) = &self {
            return Ok(value.clone());
        }

        Err(())
    }
}

impl TryInto<i64> for ScalarValue {
    type Error = ();

    fn try_into(self) -> Result<i64, Self::Error> {
        if let ScalarValue::Int(scalar) = &self {
            return Ok(scalar.clone());
        }

        Err(())
    }
}

impl TryInto<f64> for ScalarValue {
    type Error = ();

    fn try_into(self) -> Result<f64, Self::Error> {
        if let ScalarValue::Decimal(scalar) = &self {
            return Ok(scalar.clone());
        }

        Err(())
    }
}

impl TryInto<String> for ScalarValue {
    type Error = ();

    fn try_into(self) -> Result<String, Self::Error> {
        if let ScalarValue::String(scalar) = &self {
            return Ok(scalar.clone());
        }

        Err(())
    }
}
