use std::sync::Arc;

use error::Result;
use expr::Expression;
use logical_plan::{Filter, LogicalPlan, Projection, Scan};
use physical_plan::planner::PhysicalPlanner;

pub mod common;
pub mod expr;
pub mod logical_plan;
pub mod physical_plan;
pub mod prelude;
pub mod table;
pub mod error;

/// SessionContext is the entry point toy-df.
pub struct SessionContext {}

impl SessionContext {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_csv<P: FilePaths>(&self, paths: P) -> Result<DataFrame> {
        Ok(DataFrame {
            plan: Arc::new(LogicalPlan::Scan(Scan {
                source_paths: paths.file_paths().into_iter().collect(),
            })),
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
    pub plan: Arc<LogicalPlan>,
}

impl DataFrame {
    pub fn select(self, exprs: Vec<Expression>) -> DataFrame {
        DataFrame {
            plan: Arc::new(LogicalPlan::Projection(Projection {
                input: self.plan,
                exprs,
            })),
        }
    }

    pub fn filter(self, expr: Expression) -> DataFrame {
        DataFrame {
            plan: Arc::new(LogicalPlan::Filter(Filter {
                input: self.plan,
                expr,
            })),
        }
    }

    pub fn show(&self) {
        let planner = PhysicalPlanner::new();
        let physical_plan = planner.create_physical_plan(Arc::clone(&self.plan));
        println!("{:#?}", physical_plan);
    }

    pub fn logical_plan(&self) -> Arc<LogicalPlan> {
        Arc::clone(&self.plan)
    }
}
