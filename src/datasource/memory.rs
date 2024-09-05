use std::{fmt::Display, sync::Arc};

use arrow::array::RecordBatch;
use arrow_schema::SchemaRef;

use crate::{error::Result, physical_plan::plan::ExecutionPlan};

use super::TableProvider;

/// In-memory datasource.
#[derive(Debug)]
pub struct MemTable {
    schema: SchemaRef,
    data: Vec<RecordBatch>,
    description: Option<String>,
}

impl MemTable {
    // TODO: Validate RecordBatches against schema.
    pub fn new(schema: SchemaRef, data: Vec<RecordBatch>, description: Option<String>) -> Self {
        Self { schema, data, description }
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

impl Display for MemTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemTable")?;
        if let Some(description) = &self.description {
            write!(f, " ({description})")
        } else {
            Ok(())
        }
    }
}
