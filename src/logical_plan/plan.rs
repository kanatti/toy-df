use std::sync::Arc;

use crate::expr::Expression;

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
                    pp(format!("Scan {:?}", scan.source_paths), level);
                }
                LogicalPlan::Limit(_) => todo!(),
            }
        }

        describe_at(self, 0);
    }

    pub fn inputs(&self) -> Vec<Arc<LogicalPlan>> {
        match self {
            LogicalPlan::Projection(projection) => vec![projection.input.clone()],
            LogicalPlan::Filter(filter) => vec![filter.input.clone()],
            LogicalPlan::Aggregate(_) => todo!(),
            LogicalPlan::Sort(_) => todo!(),
            LogicalPlan::Join(_) => todo!(),
            LogicalPlan::Scan(_) => vec![],
            LogicalPlan::Limit(_) => todo!(),
        }
    }
}

pub struct Projection {
    pub exprs: Vec<Expression>,
    pub input: Arc<LogicalPlan>,
}

pub struct Filter {
    pub expr: Expression,
    pub input: Arc<LogicalPlan>,
}

pub struct Aggregate {}

pub struct Sort {}

pub struct Join {}

pub struct Scan {
    pub source_paths: Vec<String>,
}

pub struct Limit {}

fn pp(message: String, level: usize) {
    let padding = "  ".repeat(level);
    println!("{}{}", padding, message);
}

// Helpers for creating logical plan nodes

pub mod helpers {
    use super::*;

    pub fn projection(exprs: Vec<Expression>, input: Arc<LogicalPlan>) -> Arc<LogicalPlan> {
        Arc::new(LogicalPlan::Projection(Projection { exprs, input }))
    }

    pub fn filter(expr: Expression, input: Arc<LogicalPlan>) -> Arc<LogicalPlan> {
        Arc::new(LogicalPlan::Filter(Filter { expr, input }))
    }

    pub fn scan(source_paths: Vec<String>) -> Arc<LogicalPlan> {
        Arc::new(LogicalPlan::Scan(Scan { source_paths }))
    }
}
