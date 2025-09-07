use tracing::info;

/// Centralized system status logging
pub fn log_system_status(
    server_url: &str,
    ui_url: Option<&str>,
    db_status: &str,
) {
    info!("🖥️  Server started at: {}", server_url);
    if let Some(ui) = ui_url {
        info!("🎨 UI running at: {}", ui);
    }
    info!("💾 Database status: {}", db_status);
}
