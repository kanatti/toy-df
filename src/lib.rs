use expr::Expression;
use logical_plan::{Filter, LogicalPlan, Projection, Scan};

pub mod common;
pub mod expr;
pub mod logical_plan;
pub mod prelude;

/// SessionContext is the entry point toy-df.
pub struct SessionContext {}

impl SessionContext {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_csv<P: FilePaths>(&self, paths: P) -> Result<DataFrame> {
        Ok(DataFrame {
            plan: LogicalPlan::Scan(Scan {
                source_paths: paths.file_paths().into_iter().collect(),
            }),
        })
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

pub struct DataFrame {
    plan: LogicalPlan,
}

impl DataFrame {
    pub fn select(&self, exprs: Vec<Expression>) -> DataFrame {
        DataFrame {
            plan: LogicalPlan::Projection(Projection {

            })
        }
    }

    pub fn filter(&self, expr: Expression) -> DataFrame {
        DataFrame {
            plan: LogicalPlan::Filter(Filter {

            })
        }
    }

    pub fn show(&self) {
        println!("DataFrame");
    }
}

#[derive(Debug)]
pub struct QueryEngineError {}

pub type Result<T> = std::result::Result<T, QueryEngineError>;
