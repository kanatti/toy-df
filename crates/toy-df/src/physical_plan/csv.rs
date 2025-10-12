use crate::error::Result;

use super::plan::{ExecutionPlan, RecordBatchStream};

#[derive(Debug)]
pub struct CsvExec {}

impl CsvExec {
    pub fn new() -> Self {
        CsvExec {}
    }
}

impl ExecutionPlan for CsvExec {
    fn execute(&self) -> Result<RecordBatchStream>{
        Ok(vec![])
    }
}