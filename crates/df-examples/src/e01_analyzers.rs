use datafusion::prelude::*;
use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
use datafusion::functions_aggregate::count::count;
use datafusion::optimizer::analyzer::Analyzer;
use datafusion::common::Result;
use datafusion::datasource::provider_as_source;
use std::sync::Arc;

pub async fn run() -> Result<()> {
    let ctx = SessionContext::new();
    
    // Create test table
    let schema = Arc::new(arrow::datatypes::Schema::new(vec![
        arrow::datatypes::Field::new("id", arrow::datatypes::DataType::Int32, false),
        arrow::datatypes::Field::new("name", arrow::datatypes::DataType::Utf8, false),
        arrow::datatypes::Field::new("age", arrow::datatypes::DataType::Int32, true),
    ]));
    
    let batch = arrow::record_batch::RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(arrow::array::Int32Array::from(vec![1, 2, 3])),
            Arc::new(arrow::array::StringArray::from(vec!["Alice", "Bob", "Charlie"])),
            Arc::new(arrow::array::Int32Array::from(vec![Some(25), None, Some(30)])),
        ],
    )?;
    
    ctx.register_batch("users", batch)?;
    
    // Build a logical plan directly that needs analysis
    let table_provider = ctx.table_provider("users").await?;
    let table_source = provider_as_source(table_provider);
    
    let raw_plan = LogicalPlanBuilder::scan("users", table_source, None)?
        .filter(col("age").gt(lit("25")))? // String literal that needs type coercion
        .aggregate(vec![col("name")], vec![count(lit(1))])?
        .project(vec![count(lit(1)), col("name")])?
        .build()?;
    
    println!("=== BEFORE Analysis ===");
    println!("{}", raw_plan.display_indent());
    
    // Apply analyzer
    let session_state = ctx.state();
    let analyzer = Analyzer::new();
    let analyzed_plan = analyzer.execute_and_check(
        raw_plan, 
        session_state.config_options(),
        |_, _| {},
    )?;
    
    println!("\n=== AFTER Analysis ===");
    println!("{}", analyzed_plan.display_indent());
    
    Ok(())
}
