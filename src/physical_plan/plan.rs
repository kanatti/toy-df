use std::sync::Arc;

use arrow::array::RecordBatch;
use arrow_schema::Schema;

use crate::{datasource::TableProvider, error::Result, expr::Expression};

pub type RecordBatchStream = Vec<RecordBatch>;

pub trait ExecutionPlan: std::fmt::Debug {
    fn execute(&self) -> Result<RecordBatchStream>;
}

pub enum ExecutionError {}

#[derive(Debug)]
pub struct ScanExec {
    pub table: Arc<dyn TableProvider>
}

impl ScanExec {
    pub fn new(table: Arc<dyn TableProvider>) -> Self {
        ScanExec { table }
    }
}

impl ExecutionPlan for ScanExec {
    fn execute(&self) -> Result<RecordBatchStream> {
        Ok(vec![
            RecordBatch::new_empty(Arc::new(Schema::empty()))
        ])
    }
}

#[derive(Debug)]
pub struct FilterExec {
    pub expr: Expression,
    pub input: Arc<dyn ExecutionPlan>,
}

impl FilterExec {
    pub fn new(expr: Expression, input: Arc<dyn ExecutionPlan>) -> Self {
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
    pub exprs: Vec<Expression>,
    pub input: Arc<dyn ExecutionPlan>,
}

impl ProjectionExec {
    pub fn new(exprs: Vec<Expression>, input: Arc<dyn ExecutionPlan>) -> Self {
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
