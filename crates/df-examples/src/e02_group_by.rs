use arrow::array::{Array, ArrayRef, BooleanArray};
use arrow::datatypes::{DataType, Field};
use arrow::util::display::{ArrayFormatter, FormatOptions};
use datafusion::common::Result;
use datafusion::logical_expr::EmitTo;
use datafusion::physical_plan::aggregates::group_values::new_group_values;
use datafusion::physical_plan::display::DisplayableExecutionPlan;
use datafusion::prelude::*;
use std::sync::Arc;

fn array_to_string(array: &ArrayRef) -> String {
    let options = FormatOptions::default().with_null("null");
    let formatter = ArrayFormatter::try_new(array, &options).unwrap();
    let mut s = String::from("[");
    for i in 0..array.len() {
        if i > 0 {
            s.push_str(", ");
        }
        formatter.value(i).write(&mut s).unwrap();
    }
    s.push(']');
    s
}

fn print_kv(label: &str, value: String) {
    println!("  {:<30}: {}", label, value);
}

fn print_header(title: &str) {
    println!("\n--- {} ---", title);
}

pub async fn run() -> Result<()> {
    show_simple_groupby_plans().await?;
    group_values_boolean()?;

    Ok(())
}

async fn show_simple_groupby_plans() -> Result<()> {
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
            Arc::new(arrow::array::StringArray::from(vec![
                "A", "A", "B", "B", "C",
            ])),
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
    println!(
        "{}",
        DisplayableExecutionPlan::new(physical_plan.as_ref()).indent(true)
    );

    Ok(())
}

fn group_values_boolean() -> Result<()> {
    println!("=== GROUP VALUES BOOLEAN ===");

    let schema = Arc::new(arrow::datatypes::Schema::new(vec![Field::new(
        "bool_col",
        DataType::Boolean,
        true,
    )]));

    let mut gv = new_group_values(schema.clone())?;

    print_header("Initial Intern");

    // Create boolean array with true, false, true, null
    let array = Arc::new(BooleanArray::from(vec![
        Some(true),
        Some(false),
        Some(true),
    ])) as ArrayRef;
    let cols = &[array.clone()];

    print_kv("Input columns", array_to_string(&cols[0]));
    print_kv("gv len", gv.len().to_string());

    let mut groups = Vec::new();
    println!("  Interning columns...");
    gv.intern(cols, &mut groups)?;

    print_kv("gv len", gv.len().to_string());
    print_kv("Group IDs", format!("{:?}", groups));

    print_header("Second Intern");

    let array = Arc::new(BooleanArray::from(vec![
        Some(true),
        Some(false),
        Some(true),
        None,
    ])) as ArrayRef;
    let cols = &[array.clone()];

    print_kv("Input columns", array_to_string(&cols[0]));
    print_kv("gv len", gv.len().to_string());

    let mut groups = Vec::new();
    println!("  Interning columns...");
    gv.intern(cols, &mut groups)?;

    print_kv("gv len", gv.len().to_string());
    print_kv("Group IDs", format!("{:?}", groups));

    print_header("Emit All");
    let emitted = gv.emit(EmitTo::All)?;
    print_kv("Emitted keys (All)", array_to_string(&emitted[0]));

    print_kv("gv len", gv.len().to_string());

    print_header("Emit Again");

    // Test calling emit again on empty state
    let _emitted_empty = gv.emit(EmitTo::All)?;
    print_kv("Emitted empty", "[]".to_string());

    Ok(())
}
