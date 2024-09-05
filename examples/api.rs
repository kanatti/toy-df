use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema};
use toy_df::datasource::csv::CsvReadOptions;
use toy_df::prelude::*;
use toy_df::SessionContext;

fn main() {
    let ctx = SessionContext::new();

    // Read csv and get a DataFrame
    let schema = Schema::new(vec![
        Field::new("alias", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("age", DataType::Int16, false),
    ]);

    let read_options = CsvReadOptions::new(true, Some(Arc::new(schema)));

    let data_file = format!("{}/examples/data.csv", env!("CARGO_MANIFEST_DIR"));

    let mut df = ctx.read_csv(data_file, read_options).unwrap();

    // Operate on data frame
    df = df
        .select(vec![col("name"), col("age")])
        .filter(col("age").gt(lit(30)));

    // Filter BinaryExpression(BinaryExpression { left: Column(Column { name: "age" }), right: Literal(Int32(Some(30))), op: GreaterThan })
    //   Projection [Column(Column { name: "name" }), Column(Column { name: "age" })]
    //     Scan ["/Users/balu/Code/toy-df/examples/data.csv"]

    df.logical_plan().describe();

    // Currently just shows physical plan.
    // Ok(
    //     FilterExec {
    //         expr: BinaryExpression(
    //             BinaryExpression {
    //                 left: Column(
    //                     Column {
    //                         name: "age",
    //                     },
    //                 ),
    //                 right: Literal(
    //                     Int32(
    //                         Some(
    //                             30,
    //                         ),
    //                     ),
    //                 ),
    //                 op: GreaterThan,
    //             },
    //         ),
    //         input: ProjectionExec {
    //             exprs: [
    //                 Column(
    //                     Column {
    //                         name: "name",
    //                     },
    //                 ),
    //                 Column(
    //                     Column {
    //                         name: "age",
    //                     },
    //                 ),
    //             ],
    //             input: ScanExec {
    //                 source_paths: [
    //                     "/Users/balu/Code/toy-df/examples/data.csv",
    //                 ],
    //             },
    //         },
    //     },
    // )
    df.show();
}
