use std::env::{args, set_var};
use reauth_core::{config::Settings, initialize, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = args().collect();
    if let Some(config_path) = parse_config_path(&args)? {
        set_var("REAUTH_CONFIG", config_path);
    }

    if args.iter().any(|a| a == "--print-config") {
        let settings = Settings::new()?;
        let redacted = settings.redacted();
        let output = serde_json::to_string_pretty(&redacted)?;
        println!("{}", output);
        return Ok(());
    }

    if args.iter().any(|a| a == "--benchmark") {
        let _ = initialize().await?;
        println!("Initialization complete — exiting (benchmark mode)");
        return Ok(());
    }

    run().await
}

fn parse_config_path(args: &[String]) -> anyhow::Result<Option<String>> {
    for (idx, arg) in args.iter().enumerate() {
        if let Some(value) = arg.strip_prefix("--config=") {
            return Ok(Some(value.to_string()));
        }
        if arg == "--config" {
            let value = args
                .get(idx + 1)
                .filter(|v| !v.starts_with("--"))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("--config requires a file path"))?;
            return Ok(Some(value));
        }
    }
    Ok(None)
}
