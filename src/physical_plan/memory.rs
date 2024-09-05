use arrow::array::RecordBatch;

use crate::error::Result;

use super::plan::{ExecutionPlan, RecordBatchStream};

/// ExecutionPlan that reads in-memory data.
#[derive(Debug)]
pub struct MemoryExec {
    pub data: Vec<RecordBatch>,
}

impl ExecutionPlan for MemoryExec {
    fn execute(&self) -> Result<RecordBatchStream> {
        Ok(self.data.clone())
    }
}
