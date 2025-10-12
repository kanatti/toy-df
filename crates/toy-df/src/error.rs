use std::result;

pub type Result<T> = result::Result<T, ToyDfError>;

#[derive(Debug)]
pub enum ToyDfError {
    ExecutionError,
    PlanError,
    TableNotFound,
    // Add other error types as needed
}
