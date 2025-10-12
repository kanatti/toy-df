//! Utility functions for easily creating expressions.

use super::{Column, Expr, ScalarValue};

pub fn col(name: impl Into<String>) -> Expr {
    Expr::Column(Column::from_name(name.into()))
}

pub fn lit(value: impl  Into<ScalarValue>) -> Expr {
    Expr::Literal(value.into())
}