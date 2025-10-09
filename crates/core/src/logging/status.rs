use tracing::info;

/// Centralized system status logging
pub fn log_system_status(
    server_url: &str,
    db_status: &str,
) {
    info!("🖥️  Server started at: {}", server_url);
    info!("💾 Database status: {}", db_status);
}
