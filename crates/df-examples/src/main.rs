mod e01_analyzers;

use datafusion::common::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let example = args.get(1).map(|s| s.as_str()).unwrap_or("analyzers");
    
    match example {
        "analyzers" => e01_analyzers::run().await,
        _ => {
            println!("Available examples: analyzers");
            println!("Usage: cargo run [example_name]");
            Ok(())
        }
    }
}
