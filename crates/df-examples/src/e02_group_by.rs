use datafusion::prelude::*;
use datafusion::physical_plan::display::DisplayableExecutionPlan;
use datafusion::common::Result;
use std::sync::Arc;

pub async fn run() -> Result<()> {
    let ctx = SessionContext::new();

    // Create test table table1
    let schema = Arc::new(arrow::datatypes::Schema::new(vec![
        arrow::datatypes::Field::new("x", arrow::datatypes::DataType::Int32, false),
        arrow::datatypes::Field::new("y", arrow::datatypes::DataType::Utf8, false),
    ]));

    let batch = arrow::record_batch::RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(arrow::array::Int32Array::from(vec![1, 2, 3, 4, 5])),
            Arc::new(arrow::array::StringArray::from(vec!["A", "A", "B", "B", "C"])),
        ],
    )?;

    ctx.register_batch("table1", batch)?;

    // Execute SQL query
    let df = ctx.sql("SELECT y, SUM(x) FROM table1 GROUP BY y").await?;

    // Get logical plan
    let logical_plan = df.logical_plan().clone();
    println!("=== LOGICAL PLAN ===");
    println!("{}", logical_plan.display_indent());

    // Get physical plan
    let physical_plan = df.create_physical_plan().await?;
    println!("\n=== PHYSICAL PLAN ===");
    println!("{}", DisplayableExecutionPlan::new(physical_plan.as_ref()).indent(true));

    Ok(())
}
