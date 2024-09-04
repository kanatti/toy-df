use std::{result, sync::Arc};

use crate::logical_plan::LogicalPlan;

pub type Result<T> = result::Result<T, ExecutionError>;

pub trait ExecutionPlan {
    fn execute(&self) -> Result<RecordBatch>;
}

pub struct RecordBatch {}

pub enum ExecutionError {}

pub struct ScanExec {}

impl ExecutionPlan for ScanExec {
    fn execute(&self) -> Result<RecordBatch> {
        Ok(RecordBatch {})
    }
}

pub struct FilterExec {}

impl ExecutionPlan for FilterExec {
    fn execute(&self) -> Result<RecordBatch> {
        Ok(RecordBatch {})
    }
}

pub struct ProjectionExec {}

impl ExecutionPlan for ProjectionExec {
    fn execute(&self) -> Result<RecordBatch> {
        Ok(RecordBatch {})
    }
}

pub struct DummyExec {
    logical_plan: Arc<LogicalPlan>,
}

impl DummyExec {
    pub fn new(logical_plan: Arc<LogicalPlan>) -> Self {
        Self { logical_plan }
    }
}

impl ExecutionPlan for DummyExec {
    fn execute(&self) -> Result<RecordBatch> {
        Ok(RecordBatch {})
    }
}
