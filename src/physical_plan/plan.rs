use std::result;

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
