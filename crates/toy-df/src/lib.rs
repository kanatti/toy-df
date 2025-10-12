use std::{collections::HashMap, sync::Arc};

use datasource::{
    csv::{CsvReadOptions, CsvTable},
    TableProvider,
};
use error::{Result, ToyDfError};
use expr::Expr;
use logical_plan::{Filter, LogicalPlan, Projection, Scan};
use physical_plan::planner::PhysicalPlanner;

pub mod common;
pub mod datasource;
pub mod error;
pub mod expr;
pub mod logical_plan;
pub mod physical_plan;
pub mod prelude;

/// SessionContext is the entry point toy-df.
pub struct SessionContext {
    tables: HashMap<String, Arc<dyn TableProvider>>,
}

impl SessionContext {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn read_csv<P: FilePaths>(&self, paths: P, options: CsvReadOptions) -> Result<DataFrame> {
        Ok(DataFrame {
            plan: Arc::new(LogicalPlan::Scan(Scan {
                table: Arc::new(CsvTable {
                    source_paths: paths.file_paths().into_iter().collect(),
                }),
            })),
        })
    }

    pub fn register_table(&mut self, name: String, provider: Arc<dyn TableProvider>) -> Result<()> {
        self.tables.insert(name, provider);
        Ok(())
    }

    pub fn table(&self, name: String) -> Result<DataFrame> {
        self.tables
            .get(&name)
            .map(|provider| DataFrame {
                plan: Arc::new(LogicalPlan::Scan(Scan {
                    table: provider.clone(),
                })),
            })
            .ok_or_else(|| ToyDfError::TableNotFound)
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
    pub fn select(self, exprs: Vec<Expr>) -> DataFrame {
        DataFrame {
            plan: Arc::new(LogicalPlan::Projection(Projection {
                input: self.plan,
                exprs,
            })),
        }
    }

    pub fn filter(self, expr: Expr) -> DataFrame {
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
        let results = physical_plan.unwrap().execute();
        println!("{:?}", results);
    }

    pub fn logical_plan(&self) -> Arc<LogicalPlan> {
        Arc::clone(&self.plan)
    }
}
