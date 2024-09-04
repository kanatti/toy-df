use std::{result, sync::Arc};

use crate::expr::Expression;

pub type Result<T> = result::Result<T, ExecutionError>;

pub trait ExecutionPlan: std::fmt::Debug {
    fn execute(&self) -> Result<RecordBatch>;
}

pub struct RecordBatch {}

pub enum ExecutionError {}

#[derive(Debug)]
pub struct ScanExec {
    pub source_paths: Vec<String>,
}

impl ScanExec {
    pub fn new(source_paths: Vec<String>) -> Self {
        ScanExec { source_paths }
    }
}

impl ExecutionPlan for ScanExec {
    fn execute(&self) -> Result<RecordBatch> {
        Ok(RecordBatch {})
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
    fn execute(&self) -> Result<RecordBatch> {
        Ok(RecordBatch {})
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
    fn execute(&self) -> Result<RecordBatch> {
        Ok(RecordBatch {})
    }
}
