//! The main implementation of the `PluginManager`.

use crate::plugin::{PluginStatus, PluginStatusInfo};
use crate::{
    constants,
    error::{Error, Result},
    grpc::plugin::v1::{handshake_client::HandshakeClient, Empty},
    plugin::{Manifest, PluginInstance},
    ManagerConfig,
};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Mutex,
};
use tonic::transport::Channel;
use tracing::{error, event, info, warn, Level};

/// Represents a structured log entry, used by the log bus.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// Manages the lifecycle of all discovered plugins.
///
/// The registry of running plugins is stored in `instances`, which is
/// wrapped in `Arc<Mutex<...>>` to allow for safe concurrent access.
#[derive(Clone)]
pub struct PluginManager {
    /// The in-memory "registry" of all *active* (running) plugin instances.
    pub active_plugins: Arc<Mutex<HashMap<String, PluginInstance>>>,
    /// The path to the root plugin's directory.
    plugins_dir: PathBuf,
    /// Configuration for the manager's behavior.
    config: ManagerConfig,
}

impl PluginManager {
    /// Creates a new, empty `PluginManager`.
    pub fn new(config: ManagerConfig, plugins_dir: PathBuf) -> Self {
        Self {
            active_plugins: Arc::new(Mutex::new(HashMap::new())),
            plugins_dir,
            config,
        }
    }

    /// Gets the status of all plugins by scanning the disk and comparing
    /// with the in-memory registry of active plugins.
    pub async fn get_plugin_statuses(&self) -> Result<Vec<PluginStatusInfo>> {
        let available_manifests = self.discover_available_plugins().await?;
        let active_plugins = self.active_plugins.lock().await;

        let statuses = available_manifests
            .into_iter()
            .map(|manifest| {
                let status = if active_plugins.contains_key(&manifest.id) {
                    PluginStatus::Active
                } else {
                    PluginStatus::Inactive
                };
                PluginStatusInfo { manifest, status }
            })
            .collect();

        Ok(statuses)
    }

    /// Enables a plugin by its ID.
    /// This finds, loads, and spawns the plugin process.
    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        // 1. Check if it's already active
        {
            if self.active_plugins.lock().await.contains_key(plugin_id) {
                warn!("Plugin '{}' is already active.", plugin_id);
                return Ok(());
            }
        }

        // 2. Find its manifest on disk
        let manifest = self
            .find_manifest_by_id(plugin_id)
            .await?
            .ok_or_else(|| Error::PluginNotFound(plugin_id.to_string()))?;

        // 3. Determine if we are in dev or prod
        let is_dev_run = std::env::current_exe().map_or(false, |p| {
            p.ancestors()
                .any(|p| p.ends_with(constants::DEV_ENVIRONMENT_DIR))
        });

        // 4. Call the internal load/spawn logic
        self.load_and_spawn_plugin(manifest, is_dev_run).await
    }

    /// Disables a running plugin by its ID.
    /// This kills the process and removes it from the active registry.
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut active_plugins = self.active_plugins.lock().await;

        if let Some(mut instance) = active_plugins.remove(plugin_id) {
            info!("Disabling plugin '{}'...", plugin_id);
            if let Err(e) = instance.process.kill().await {
                error!("Failed to kill plugin process for '{}': {}", plugin_id, e);
            }
            info!("Plugin '{}' has been disabled.", plugin_id);
            Ok(())
        } else {
            Err(Error::PluginNotActive(plugin_id.to_string()))
        }
    }

    /// Returns the gRPC channel for a specific, *active* plugin.
    /// Returns `None` if the plugin is not currently running.
    pub async fn get_active_plugin_channel(&self, plugin_id: &str) -> Option<Channel> {
        let active_plugins = self.active_plugins.lock().await;
        active_plugins
            .get(plugin_id)
            .map(|instance| instance.grpc_channel.clone())
    }

    /// Returns a Vec of (Manifest, Channel) for all currently *active* plugins.
    /// This is used by the EventGateway to fan-out events.
    pub async fn get_all_active_plugins(&self) -> Vec<(Manifest, Channel)> {
        let active_plugins = self.active_plugins.lock().await;
        active_plugins
            .values()
            .map(|instance| (instance.manifest.clone(), instance.grpc_channel.clone()))
            .collect()
    }

    async fn discover_available_plugins(&self) -> Result<Vec<Manifest>> {
        let entries = match std::fs::read_dir(&self.plugins_dir) {
            Ok(entries) => entries,
            Err(_) => {
                error!("Plugins directory not found: {:?}", self.plugins_dir);
                return Err(Error::PluginsDirNotFound(self.plugins_dir.clone()));
            }
        };

        let mut manifests = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join(constants::MANIFEST_FILENAME);
            if !manifest_path.exists() {
                continue;
            }

            let manifest_str = std::fs::read_to_string(manifest_path)?;
            let manifest: Manifest = serde_json::from_str(&manifest_str)
                .map_err(|e| Error::ManifestParse { path, source: e })?;
            manifests.push(manifest);
        }
        Ok(manifests)
    }

    /// Helper to find a single manifest by its ID.
    async fn find_manifest_by_id(&self, plugin_id: &str) -> Result<Option<Manifest>> {
        Ok(self
            .discover_available_plugins()
            .await?
            .into_iter()
            .find(|m| m.id == plugin_id))
    }

    /// Internal function to load, spawn, and handshake a single plugin.
    async fn load_and_spawn_plugin(&self, manifest: Manifest, is_dev_run: bool) -> Result<()> {
        let executable_from_json = if cfg!(target_os = "windows") {
            &manifest.executable.windows_amd64
        } else {
            &manifest.executable.linux_amd64
        };

        let executable_path = if is_dev_run {
            PathBuf::from(executable_from_json)
        } else {
            // In production, path is relative to its own manifest.json
            self.plugins_dir
                .join(&manifest.id)
                .join(executable_from_json)
        };

        self.spawn_and_handshake(manifest, executable_path).await
    }

    // This function is identical to your previous version.
    async fn spawn_and_handshake(
        &self,
        manifest: Manifest,
        executable_path: PathBuf,
    ) -> Result<()> {
        info!("Attempting to spawn plugin from: {:?}", &executable_path);
        let mut child = Command::new(&executable_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::PluginSpawn {
                name: manifest.name.clone(),
                path: executable_path,
                source: e,
            })?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let mut stdout_reader = BufReader::new(stdout).lines();

        let handshake_timeout_duration = Duration::from_secs(self.config.handshake_timeout_secs);
        let handshake_result = tokio::time::timeout(handshake_timeout_duration, async {
            if let Ok(Some(line)) = stdout_reader.next_line().await {
                let parts: Vec<&str> = line.split(constants::HANDSHAKE_DELIMITER).collect();
                if parts.len() == constants::HANDSHAKE_PARTS_COUNT
                    && parts[4] == constants::HANDSHAKE_PROTOCOL_TYPE
                {
                    let addr = format!("{}://{}", constants::HANDSHAKE_URL_SCHEME, parts[3]);
                    if let Ok(channel) = Channel::from_shared(addr).unwrap().connect().await {
                        return Ok(channel); // Handshake seems valid, return channel for verification.
                    }
                }
            }
            Err(Error::HandshakeInvalid {
                name: manifest.name.clone(),
                reason: "Invalid format or no line received".into(),
            })
        })
        .await;

        match handshake_result {
            Ok(Ok(channel)) => {
                let mut client = HandshakeClient::new(channel.clone());
                client
                    .get_plugin_info(Empty {})
                    .await
                    .map_err(|e| Error::GrpcVerification {
                        name: manifest.name.clone(),
                        source: e,
                    })?;

                info!("Successfully connected to plugin '{}'", manifest.name);

                let instance = PluginInstance {
                    process: child,
                    manifest: manifest.clone(),
                    grpc_channel: channel,
                };

                // Add the new instance to the active registry
                self.active_plugins
                    .lock()
                    .await
                    .insert(manifest.id.clone(), instance);

                self.spawn_log_forwarders(manifest.name, stdout_reader, stderr);
                Ok(())
            }
            _ => {
                error!(
                    "Plugin {} did not handshake in time or handshake was invalid.",
                    manifest.name
                );
                child.kill().await?;
                Err(Error::HandshakeTimeout(manifest.name.clone()))
            }
        }
    }

    fn spawn_log_forwarders<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
        &self,
        name: String,
        stdout_reader: tokio::io::Lines<BufReader<R>>,
        stderr: tokio::process::ChildStderr,
    ) {
        // Handle stdout (handshake output)
        let stdout_plugin_name = name.clone();
        tokio::spawn(async move {
            let mut lines = stdout_reader;
            while let Ok(Some(line)) = lines.next_line().await {
                event!(
                    target: "plugin_handshake",
                    Level::INFO,
                    plugin_name = %stdout_plugin_name,
                    "{line}"
                );
            }
        });

        // Handle stderr (plugin's structured JSON logs)
        let stderr_plugin_name = name.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            use tracing::{debug, error, info, trace, warn};

            let mut lines = BufReader::new(stderr).lines();

            while let Ok(Some(line)) = lines.next_line().await {
                match serde_json::from_str::<crate::plugin::PluginLogLine>(&line) {
                    Ok(mut log_line) => {
                        // Fallback: ensure message is present
                        if log_line.message.is_empty() {
                            if let Some(msg) = log_line
                                .fields
                                .remove("message")
                                .or(log_line.fields.remove("msg"))
                            {
                                log_line.message = msg.to_string().trim_matches('"').to_string();
                            }
                        }

                        // Normalize log level
                        let level = log_line.level.to_uppercase();

                        // Match on log level to call the correct macro variant
                        match level.as_str() {
                            "TRACE" => trace!(
                                target: "plugin_log",
                                plugin_name = %stderr_plugin_name,
                                plugin_target = %log_line.target,
                                plugin_fields = ?log_line.fields,
                                "{}",
                                log_line.message
                            ),
                            "DEBUG" => debug!(
                                target: "plugin_log",
                                plugin_name = %stderr_plugin_name,
                                plugin_target = %log_line.target,
                                plugin_fields = ?log_line.fields,
                                "{}",
                                log_line.message
                            ),
                            "INFO" => info!(
                                target: "plugin_log",
                                plugin_name = %stderr_plugin_name,
                                plugin_target = %log_line.target,
                                plugin_fields = ?log_line.fields,
                                "{}",
                                log_line.message
                            ),
                            "WARN" | "WARNING" => warn!(
                                target: "plugin_log",
                                plugin_name = %stderr_plugin_name,
                                plugin_target = %log_line.target,
                                plugin_fields = ?log_line.fields,
                                "{}",
                                log_line.message
                            ),
                            "ERROR" => error!(
                                target: "plugin_log",
                                plugin_name = %stderr_plugin_name,
                                plugin_target = %log_line.target,
                                plugin_fields = ?log_line.fields,
                                "{}",
                                log_line.message
                            ),
                            _ => info!(
                                target: "plugin_log",
                                plugin_name = %stderr_plugin_name,
                                plugin_target = %log_line.target,
                                plugin_fields = ?log_line.fields,
                                "{}",
                                log_line.message
                            ),
                        }
                    }
                    Err(_) => {
                        // Non-JSON line (panic or plain stderr)
                        error!(
                            target: "plugin_panic",
                            plugin_name = %stderr_plugin_name,
                            "Invalid log line or panic: {}",
                            line
                        );
                    }
                }
            }
        });
    }
}
