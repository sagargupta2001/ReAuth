use std::env::{args, set_var};
use std::fs;
use std::path::PathBuf;
use reauth_core::{config::Settings, initialize, run};

const HELP_TEXT: &str = r#"ReAuth Core

Usage:
  reauth_core [flags]

Flags:
  --help, -h        Show this help text and exit
  --config <path>   Load config from a specific file
  --print-config    Print resolved config (secrets redacted) and exit
  --check-config    Validate resolved config and exit
  --init-config     Write a commented reauth.toml template next to the binary
  --benchmark       Run initialization and migrations, then exit
"#;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", HELP_TEXT);
        return Ok(());
    }

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

    if args.iter().any(|a| a == "--check-config") {
        let _settings = Settings::new()?;
        println!("Config OK");
        return Ok(());
    }

    if args.iter().any(|a| a == "--init-config") {
        let target_path = resolve_init_config_path()?;
        write_config_template(&target_path)?;
        println!("Initialized config template at {}", target_path.display());
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

fn resolve_init_config_path() -> anyhow::Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve executable directory"))?;
    Ok(exe_dir.join("reauth.toml"))
}

fn write_config_template(dest_path: &PathBuf) -> anyhow::Result<()> {
    let template = include_str!("../../../config/reauth.toml.template");
    if dest_path.exists() {
        return Err(anyhow::anyhow!(
            "Config already exists at {}",
            dest_path.display()
        ));
    }
    fs::write(dest_path, template)?;
    Ok(())
}
