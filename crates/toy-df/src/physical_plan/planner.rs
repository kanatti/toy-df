use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{
    error::{Result, ToyDfError},
    logical_plan::{self, LogicalPlan},
};

use super::plan::{ExecutionPlan, FilterExec, ProjectionExec};

pub struct PhysicalPlanner {}

impl PhysicalPlanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_physical_plan(
        &self,
        logical_plan: Arc<LogicalPlan>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // For creating a physical node, we need to know all the children nodes first.
        // This means we need a bottom-up traversal of the logical_plan tree.

        let root = physical_node_ref(logical_plan, None);

        // Do a BFS to flatten the tree and then walk the tree in reverse order.
        let mut stack = vec![root];
        let mut peek = 0;
        while peek < stack.len() {
            let node = Rc::clone(&stack[peek]);
            for child in node.borrow().logical_plan.inputs() {
                let node = physical_node_ref(child, Some(Rc::clone(&node)));
                stack.push(node);
            }
            peek += 1;
        }

        let mut root_plan = None;

        // Walk the tree from leaf up.
        for node in stack.into_iter().rev() {
            // We expect all children to be converted to physical plan.
            // TODO: Throw proper error.
            assert_eq!(
                node.borrow().children_expected,
                node.borrow().children_converted.len()
            );

            // convert the plan.
            let physical_plan = convert_to_plan(Rc::clone(&node));

            // update parent.
            if let Some(ref parent) = node.borrow().parent {
                parent.borrow_mut().children_converted.push(physical_plan);
            } else {
                root_plan = Some(physical_plan);
            }
        }

        root_plan.map_or(Err(ToyDfError::PlanError), |plan| Ok(plan))
    }
}

/// An intermediate holder for generating execution plan.
#[derive(Debug)]
struct PhysicalNode {
    pub children_expected: usize,
    pub children_converted: Vec<Arc<dyn ExecutionPlan>>,
    pub parent: Option<PhysicalNodeRef>,
    pub logical_plan: Arc<LogicalPlan>,
}

// We only use single thread now.
// Move to Arc<Mutex<>> if multi-threaded in future.
type PhysicalNodeRef = Rc<RefCell<PhysicalNode>>;

fn physical_node_ref(
    logical_plan: Arc<LogicalPlan>,
    parent: Option<PhysicalNodeRef>,
) -> PhysicalNodeRef {
    Rc::new(RefCell::new(PhysicalNode {
        children_expected: logical_plan.inputs().len(),
        children_converted: vec![],
        parent,
        logical_plan,
    }))
}

fn convert_to_plan(node: PhysicalNodeRef) -> Arc<dyn ExecutionPlan> {
    let node = node.borrow();
    match node.logical_plan.as_ref() {
        LogicalPlan::Scan(scan) => convert_scan(scan),
        LogicalPlan::Filter(filter) => {
            let input = node.children_converted.get(0).unwrap();
            convert_filter(filter, Arc::clone(input))
        }
        LogicalPlan::Projection(projection) => {
            let input = node.children_converted.get(0).unwrap();
            convert_projection(projection, Arc::clone(input))
        }
        LogicalPlan::Aggregate(_) => todo!(),
        LogicalPlan::Sort(_) => todo!(),
        LogicalPlan::Join(_) => todo!(),
        LogicalPlan::Limit(_) => todo!(),
    }
}

pub fn convert_scan(scan: &logical_plan::Scan) -> Arc<dyn ExecutionPlan> {
    return scan.table.scan().unwrap();
}

pub fn convert_filter(
    filter: &logical_plan::Filter,
    input: Arc<dyn ExecutionPlan>,
) -> Arc<dyn ExecutionPlan> {
    // TODO: Fix clone, move to PhysicalExpr
    return Arc::new(FilterExec::new(filter.expr.clone(), input));
}

pub fn convert_projection(
    projection: &logical_plan::Projection,
    input: Arc<dyn ExecutionPlan>,
) -> Arc<dyn ExecutionPlan> {
    return Arc::new(ProjectionExec::new(projection.exprs.clone(), input));
}

#[derive(Debug)]
pub struct PlannerError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasource::csv::CsvTable;
    use crate::logical_plan::helpers::*;
    use crate::prelude::*;

    #[test]
    fn test_create_physical_plan() {
        let scan = scan(Arc::new(CsvTable {
            source_paths: vec![],
        }));
        let filter = filter(col("a").gt(lit(1)), scan);
        let projection = projection(vec![col("a"), col("b")], filter);

        let physical_planner = PhysicalPlanner::new();
        let physical_plan = physical_planner.create_physical_plan(projection);
        assert!(physical_plan.is_ok());
    }
}
