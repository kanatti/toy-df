use std::{fmt::Display, sync::Arc};

use arrow_schema::SchemaRef;

use crate::{
    error::Result,
    physical_plan::{csv::CsvExec, plan::ExecutionPlan},
};

use super::TableProvider;

pub struct CsvReadOptions {
    pub has_header: bool,
    pub schema: Option<SchemaRef>,
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
        Ok(Arc::new(CsvExec {}))
    }
}

impl Display for CsvTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CSV: [{}]", self.source_paths.join(", "))
    }
}
