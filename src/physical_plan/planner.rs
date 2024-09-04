use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::logical_plan::LogicalPlan;

use super::plan::{DummyExec, ExecutionPlan};

pub type Result<T> = std::result::Result<T, PlannerError>;

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
            let node = stack[peek].clone();
            for child in node.borrow().logical_plan.inputs() {
                let node = physical_node_ref(child, Some(node.clone()));
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
            let physical_plan = convert_to_plan(node.clone());

            // update parent.
            if let Some(ref parent) = node.borrow().parent {
                parent.borrow_mut().children_converted.push(physical_plan);
            } else {
                root_plan = Some(physical_plan);
            }
        }

        root_plan.map_or(Err(PlannerError {}), |plan| Ok(plan))
    }
}

/// An intermediate holder for generating execution plan.
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

// TODO: Actual conversion logic.
fn convert_to_plan(node: PhysicalNodeRef) -> Arc<dyn ExecutionPlan> {
    Arc::new(DummyExec::new(node.borrow().logical_plan.clone()))
}

pub struct PlannerError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logical_plan::helpers::*;
    use crate::prelude::*;

    #[test]
    fn test_create_physical_plan() {
        let scan = scan(vec![]);
        let filter = filter(col("a").gt(lit(1)), scan);
        let projection = projection(vec![col("a"), col("b")], filter);

        let physical_planner = PhysicalPlanner::new();
        let physical_plan = physical_planner.create_physical_plan(projection);
        assert!(physical_plan.is_ok());
    }
}
