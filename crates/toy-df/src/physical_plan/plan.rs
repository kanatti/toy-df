use std::sync::Arc;

use arrow::array::RecordBatch;
use arrow_schema::Schema;

use crate::{error::Result, expr::Expr};

pub type RecordBatchStream = Vec<RecordBatch>;

pub trait ExecutionPlan: std::fmt::Debug {
    fn execute(&self) -> Result<RecordBatchStream>;
}

#[derive(Debug)]
pub struct FilterExec {
    pub expr: Expr,
    pub input: Arc<dyn ExecutionPlan>,
}

impl FilterExec {
    pub fn new(expr: Expr, input: Arc<dyn ExecutionPlan>) -> Self {
        FilterExec { expr, input }
    }
}

impl ExecutionPlan for FilterExec {
    fn execute(&self) -> Result<RecordBatchStream> {
        Ok(vec![
            RecordBatch::new_empty(Arc::new(Schema::empty()))
        ])
    }
}

#[derive(Debug)]
pub struct ProjectionExec {
    pub exprs: Vec<Expr>,
    pub input: Arc<dyn ExecutionPlan>,
}

impl ProjectionExec {
    pub fn new(exprs: Vec<Expr>, input: Arc<dyn ExecutionPlan>) -> Self {
        ProjectionExec { exprs, input }
    }
}

impl ExecutionPlan for ProjectionExec {
    fn execute(&self) -> Result<RecordBatchStream> {
        Ok(vec![
            RecordBatch::new_empty(Arc::new(Schema::empty()))
        ])
    }
}
