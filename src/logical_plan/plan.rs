use crate::FilePaths;

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

pub struct Projection{}

pub struct Filter {}

pub struct Aggregate {}

pub struct Sort {}

pub struct Join {}

pub struct Scan {
    pub source_paths: Vec<String>
}

pub struct Limit {}
