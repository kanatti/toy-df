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
                    describe_at(&projection.input, level);
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
    // pad by level and print
    let padding = " ".repeat(level);
    println!("{}{}", padding, message);
}
