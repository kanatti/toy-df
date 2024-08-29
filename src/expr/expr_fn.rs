//! Utility functions for easily creating expressions.

use super::{Column, Expression, ScalarValue};

pub fn col(name: impl Into<String>) -> Expression {
    Expression::Column(Column::from_name(name.into()))
}

pub fn lit(value: impl  Into<ScalarValue>) -> Expression {
    Expression::Literal(value.into())
}