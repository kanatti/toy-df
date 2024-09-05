use std::sync::Arc;

use arrow::array::RecordBatch;
use arrow_schema::SchemaRef;

use crate::{error::Result, physical_plan::plan::ExecutionPlan};

use super::TableProvider;

/// In-memory datasource.
#[derive(Debug)]
pub struct MemTable {
    schema: SchemaRef,
    data: Vec<RecordBatch>
}

impl MemTable {
    // TODO: Validate RecordBatches against schema.
    pub fn new(schema: SchemaRef, data: Vec<RecordBatch>) -> Self {
        Self { schema, data }
    }
}

impl TableProvider for MemTable {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
    
    fn scan(&self) -> Result<Arc<dyn ExecutionPlan>> {
        todo!()
    }
}
