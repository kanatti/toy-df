use std::sync::Arc;

use crate::logical_plan::LogicalPlan;

use super::plan::ExecutionPlan;

pub type Result<T> = std::result::Result<T, PlannerError>;

pub struct PhysicalPlanner {}

impl PhysicalPlanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_physical_plan(
        &self,
        logical_plan: Arc<LogicalPlan>,
    ) -> Result<()> {
        // For creating a physical node, we need to know all the children nodes first.
        // This means we need a bottom-up traversal of the logical_plan tree.

        // Do a BFS to flatten the tree and then walk the tree in reverse order.
        let root = logical_plan;
        let mut stack = vec![root];
        let mut peek = 0;
        while peek < stack.len() {
            let node = &stack[peek];
            for child in node.inputs() {
                stack.push(child);
            }
            peek += 1;
        }

        Ok(())
    }
}

pub enum PlannerError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::logical_plan::helpers::*;

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
