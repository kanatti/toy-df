use std::sync::Arc;

use arrow_schema::SchemaRef;

use crate::{error::Result, physical_plan::plan::ExecutionPlan};

pub trait TableProvider : std::fmt::Debug  {
    fn schema(&self) -> SchemaRef;
    fn scan(&self) -> Result<Arc<dyn ExecutionPlan>>;
}

pub mod csv;
pub mod memory;