use std::sync::Arc;

use crate::{datasource::TableProvider, expr::Expr};

#[derive(Debug)]
pub enum LogicalPlan {
    Projection(Projection),
    Filter(Filter),
    Aggregate(Aggregate),
    Sort(Sort),
    Join(Join),
    Scan(Scan),
    Limit(Limit),
    // Add more plan types as needed
}

impl LogicalPlan {
    pub fn describe(&self) {
        fn describe_at(plan: &LogicalPlan, level: usize) {
            match plan {
                LogicalPlan::Projection(projection) => {
                    pp(format!("Projection {:?}", projection.exprs), level);
                    describe_at(&projection.input, level + 1);
                }
                LogicalPlan::Filter(filter) => {
                    pp(format!("Filter {:?}", filter.expr), level);
                    describe_at(&filter.input, level + 1);
                }
                LogicalPlan::Aggregate(_) => todo!(),
                LogicalPlan::Sort(_) => todo!(),
                LogicalPlan::Join(_) => todo!(),
                LogicalPlan::Scan(scan) => {
                    pp(format!("Scan {}", &scan.table), level);
                }
                LogicalPlan::Limit(_) => todo!(),
            }
        }

        describe_at(self, 0);
    }

    pub fn inputs(&self) -> Vec<Arc<LogicalPlan>> {
        match self {
            LogicalPlan::Projection(projection) => vec![Arc::clone(&projection.input)],
            LogicalPlan::Filter(filter) => vec![Arc::clone(&filter.input)],
            LogicalPlan::Aggregate(_) => todo!(),
            LogicalPlan::Sort(_) => todo!(),
            LogicalPlan::Join(_) => todo!(),
            LogicalPlan::Scan(_) => vec![],
            LogicalPlan::Limit(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct Projection {
    pub exprs: Vec<Expr>,
    pub input: Arc<LogicalPlan>,
}

#[derive(Debug)]
pub struct Filter {
    pub expr: Expr,
    pub input: Arc<LogicalPlan>,
}

#[derive(Debug)]
pub struct Aggregate {}

#[derive(Debug)]
pub struct Sort {}

#[derive(Debug)]
pub struct Join {}

#[derive(Debug)]
pub struct Scan {
    pub table: Arc<dyn TableProvider>
}

#[derive(Debug)]
pub struct Limit {}

fn pp(message: String, level: usize) {
    let padding = "  ".repeat(level);
    println!("{}{}", padding, message);
}

// Helpers for creating logical plan nodes

pub mod helpers {
    use super::*;

    pub fn projection(exprs: Vec<Expr>, input: Arc<LogicalPlan>) -> Arc<LogicalPlan> {
        Arc::new(LogicalPlan::Projection(Projection { exprs, input }))
    }

    pub fn filter(expr: Expr, input: Arc<LogicalPlan>) -> Arc<LogicalPlan> {
        Arc::new(LogicalPlan::Filter(Filter { expr, input }))
    }

    pub fn scan(table: Arc<dyn TableProvider>) -> Arc<LogicalPlan> {
        Arc::new(LogicalPlan::Scan(Scan { table }))
    }
}
