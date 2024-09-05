use std::sync::Arc;

use arrow_schema::SchemaRef;

use crate::{error::Result, physical_plan::plan::ExecutionPlan};

use super::TableProvider;

pub struct CsvReadOptions {
    pub has_header: bool,
    pub schema: Option<SchemaRef>
}

impl CsvReadOptions {
    pub fn new(has_header: bool, schema: Option<SchemaRef>) -> Self {
        Self { has_header, schema }
    }
}

#[derive(Debug)]
pub struct CsvTable {
    pub source_paths: Vec<String>,
}

impl TableProvider for CsvTable {
    fn schema(&self) -> SchemaRef {
        todo!()
    }
    
    fn scan(&self) -> Result<Arc<dyn ExecutionPlan>> {
        todo!()
    }
}
