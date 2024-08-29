use query_engine::SessionContext;
use query_engine::prelude::*;

fn main() {
    let ctx = SessionContext::new();
    // Read csv and get a DataFrame
    let mut df = ctx.read_csv("path/to/data.csv").unwrap();
    // Operate on data frame
    df = df
        .select(vec![col("a"), col("b")])
        .filter(col("a").gt(lit(10)));
    // Collect
    df.show();
}
