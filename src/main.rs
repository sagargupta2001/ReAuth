use rand::distr::{Alphanumeric, SampleString};
use reauth::adapters::persistence::sqlite_realm_repository::SqliteRealmRepository;
use reauth::adapters::persistence::sqlite_user_repository::SqliteUserRepository;
use reauth::bootstrap::database::initialize_database;
use reauth::bootstrap::seed::history::SeedHistory;
use reauth::constants::DEFAULT_REALM_NAME;
use reauth::domain::crypto::HashedPassword;
use reauth::ports::realm_repository::RealmRepository;
use reauth::ports::user_repository::UserRepository;
use reauth::{adapters::run_migrations, config::Settings, initialize, run};
use std::env::{args, set_var};
use std::fs;
use std::path::PathBuf;

const HELP_TEXT: &str = r#"ReAuth

Usage:
  reauth [flags]

Flags:
  --help, -h        Show this help text and exit
  --config <path>   Load config from a specific file
  --print-config    Print resolved config (secrets redacted) and exit
  --check-config    Validate resolved config and exit
  --init-config     Write a commented reauth.toml template next to the binary
  --seed-only       Run migrations + seeding, then exit
  --seed-status     Print applied seeders and exit
  --benchmark       Run initialization and migrations, then exit

Admin:
  reauth admin reset-password --user <username> [--realm <realm>] [--password <password>]
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

    if let Some(admin_command) = parse_admin_command(&args)? {
        return handle_admin_command(admin_command).await;
    }

    if args.iter().any(|a| a == "--print-config") {
        let settings = Settings::new()?;
        let redacted = settings.redacted();
        let output = serde_json::to_string_pretty(&redacted)?;
        println!("{}", output);
        return Ok(());
    }

    if args.iter().any(|a| a == "--check-config") {
        let settings = Settings::new()?;
        if let Some((public_origin, bind_origins)) = settings.public_url_mismatch() {
            eprintln!(
                "Warning: server.public_url origin ({}) does not match bind origin(s) {:?}. This may break cookies or redirect URIs.",
                public_origin, bind_origins
            );
        }
        println!("Config OK");
        return Ok(());
    }

    if args.iter().any(|a| a == "--seed-only") {
        let _ = initialize().await?;
        println!("Seeding complete — exiting (seed-only mode)");
        return Ok(());
    }

    if args.iter().any(|a| a == "--seed-status") {
        let settings = Settings::new()?;
        let db = initialize_database(&settings).await?;
        if let Err(err) = run_migrations(db.as_ref()).await {
            eprintln!("Migration warning: {}", err);
        }
        let history = SeedHistory::new(db.as_ref());
        let records = history.list_all().await?;
        if records.is_empty() {
            println!("No seed history found.");
        } else {
            for record in records {
                println!(
                    "{} v{} ({}): {}",
                    record.name, record.version, record.checksum, record.applied_at
                );
            }
        }
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

enum AdminCommand {
    ResetPassword {
        realm: String,
        username: String,
        password: Option<String>,
    },
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

fn parse_admin_command(args: &[String]) -> anyhow::Result<Option<AdminCommand>> {
    let Some(admin_idx) = args.iter().position(|arg| arg == "admin") else {
        return Ok(None);
    };
    let command = args.get(admin_idx + 1).map(String::as_str);
    if command != Some("reset-password") {
        return Ok(None);
    }

    let username = parse_arg_value(args, "--user")
        .ok_or_else(|| anyhow::anyhow!("--user is required for admin reset-password"))?;
    let realm = parse_arg_value(args, "--realm").unwrap_or_else(|| DEFAULT_REALM_NAME.to_string());
    let password = parse_arg_value(args, "--password");

    Ok(Some(AdminCommand::ResetPassword {
        realm,
        username,
        password,
    }))
}

fn parse_arg_value(args: &[String], key: &str) -> Option<String> {
    for (idx, arg) in args.iter().enumerate() {
        if let Some(value) = arg.strip_prefix(&format!("{}=", key)) {
            return Some(value.to_string());
        }
        if arg == key {
            return args.get(idx + 1).filter(|v| !v.starts_with("--")).cloned();
        }
    }
    None
}

async fn handle_admin_command(command: AdminCommand) -> anyhow::Result<()> {
    match command {
        AdminCommand::ResetPassword {
            realm,
            username,
            password,
        } => {
            let (password, generated) = match password {
                Some(value) => (value, false),
                None => (Alphanumeric.sample_string(&mut rand::rng(), 16), true),
            };
            admin_reset_password(&realm, &username, &password).await?;
            if generated {
                println!("Generated password: {}", password);
            }
            println!(
                "Password reset for user '{}' in realm '{}'.",
                username, realm
            );
            Ok(())
        }
    }
}

async fn admin_reset_password(
    realm_name: &str,
    username: &str,
    new_password: &str,
) -> anyhow::Result<()> {
    let settings = Settings::new()?;
    let db = initialize_database(&settings).await?;
    if let Err(err) = run_migrations(db.as_ref()).await {
        eprintln!("Migration warning: {}", err);
    }

    let realm_repo = SqliteRealmRepository::new(db.clone());
    let user_repo = SqliteUserRepository::new(db.clone());

    let realm = realm_repo
        .find_by_name(realm_name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Realm '{}' not found", realm_name))?;
    let mut user = user_repo
        .find_by_username(&realm.id, username)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User '{}' not found", username))?;

    let hashed_password = HashedPassword::new(new_password)?;
    user.hashed_password = hashed_password.as_str().to_string();
    user_repo.update(&user, None).await?;

    Ok(())
}

fn resolve_init_config_path() -> anyhow::Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve executable directory"))?;
    Ok(exe_dir.join("reauth.toml"))
}

fn write_config_template(dest_path: &PathBuf) -> anyhow::Result<()> {
    let template = include_str!("../config/reauth.toml.template");
    if dest_path.exists() {
        return Err(anyhow::anyhow!(
            "Config already exists at {}",
            dest_path.display()
        ));
    }
    fs::write(dest_path, template)?;
    Ok(())
}
