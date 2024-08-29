//! Logical expressions used in query engine.

use crate::common::scalar::ScalarValue;

pub mod expr_fn;

/// Expression Tree.
/// Used in SELECT clauses, WHERE predicates etc.
pub enum Expression {
    Literal(ScalarValue),
    Column(Column),
    BinaryExpression(Box<BinaryExpression>), // Boxed for indirecting recursive infinite size.
    ScalarFunction(ScalarFunction),
    AggregateFunction(AggregateFunction),
}

impl Expression {
    pub fn gt(self, rhs: Expression) -> Expression {
        Expression::BinaryExpression(Box::new(BinaryExpression {
            left: self,
            right: rhs,
            op: BinaryOperator::GreaterThan,
        }))
    }
}

pub struct Column {
    pub name: String,
}

impl Column {
    pub fn from_name(name: String) -> Self {
        Column { name }
    }
}

pub struct BinaryExpression {
    pub left: Expression,
    pub right: Expression,
    pub op: BinaryOperator,
}

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

pub struct ScalarFunction {
    pub name: String,
    pub args: Vec<Expression>,
}

pub struct AggregateFunction {
    pub name: String,
    pub args: Vec<Expression>,
}
