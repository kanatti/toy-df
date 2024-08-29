use toy_df::prelude::*;
use toy_df::SessionContext;

fn main() {
    let ctx = SessionContext::new();
    // Read csv and get a DataFrame
    let mut df = ctx.read_csv("path/to/data.csv").unwrap();
    // Operate on data frame
    df = df
        .select(vec![col("a"), col("b")])
        .filter(col("a").gt(lit(10)));
    df.logical_plan().describe();
    // Collect
    df.show();
}
