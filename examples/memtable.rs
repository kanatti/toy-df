use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema};
use toy_df::datasource::memory::MemTable;
use toy_df::prelude::*;
use toy_df::SessionContext;

fn main() {
    let mut ctx = SessionContext::new();

    let schema = Schema::new(vec![
        Field::new("alias", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("age", DataType::Int16, false),
    ]);

    let mem_table = MemTable::new(Arc::new(schema), vec![]);

    // Register in-memory table.
    ctx.register_table("users".to_string(), Arc::new(mem_table)).unwrap();

    // DataFrame wrapped on in-memory table
    let mut df = ctx.table("users".to_string()).unwrap();

    // Operate on data frame
    df = df
        .select(vec![col("name"), col("age")])
        .filter(col("age").gt(lit(30)));

    df.logical_plan().describe();

    df.show();
}
