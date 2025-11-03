use std::env::args;
use reauth_core::{initialize, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = args().collect();

    if args.iter().any(|a| a == "--benchmark") {
        let _ = initialize().await?;
        println!("Initialization complete — exiting (benchmark mode)");
        return Ok(());
    }

    run().await
}
