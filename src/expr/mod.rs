//! Logical expressions used in query engine.

use crate::common::scalar::ScalarValue;

pub mod expr_fn;

/// Expression Tree.
/// Used in SELECT clauses, WHERE predicates etc.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(ScalarValue),
    Column(Column),
    BinaryExpr(Box<BinaryExpr>), // Boxed for indirecting recursive infinite size.
    ScalarFunction(ScalarFunction),
    AggregateFunction(AggregateFunction),
}

impl Expr {
    pub fn gt(self, rhs: Expr) -> Expr {
        Expr::BinaryExpr(Box::new(BinaryExpr {
            left: self,
            right: rhs,
            op: BinaryOperator::GreaterThan,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
}

impl Column {
    pub fn from_name(name: String) -> Self {
        Column { name }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Expr,
    pub right: Expr,
    pub op: BinaryOperator,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct ScalarFunction {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct AggregateFunction {
    pub name: String,
    pub args: Vec<Expr>,
}
