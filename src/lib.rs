use expr::Expression;

pub mod expr;
pub mod logical_plan;
pub mod prelude;
pub mod common;

/// SessionContext is the entry point to query-engine.
pub struct SessionContext {}

impl SessionContext {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_csv<P: FilePaths>(&self , paths: P) -> Result<DataFrame> {
        Ok(DataFrame {})
    }
}

pub trait FilePaths {
    fn file_paths(&self) -> impl IntoIterator<Item = String>;
}

impl FilePaths for String {
    fn file_paths(&self) -> impl IntoIterator<Item = String> {
        return std::iter::once(self.clone());
    }
}

impl FilePaths for &str {
    fn file_paths(&self) -> impl IntoIterator<Item = String> {
        return std::iter::once((*self).to_string());
    }
}

pub struct DataFrame {}

impl DataFrame {
    pub fn select(&self, exprs: Vec<Expression>) -> DataFrame {
        DataFrame {}
    }

    pub fn filter(&self, expr: Expression) -> DataFrame {
        DataFrame {}
    }

    pub fn show(&self) {
        println!("DataFrame");
    }
}

#[derive(Debug)]
pub struct QueryEngineError {}

pub type Result<T> = std::result::Result<T, QueryEngineError>;
