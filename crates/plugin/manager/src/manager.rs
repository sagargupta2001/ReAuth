//! The main implementation of the `PluginManager`.

use crate::{
    error::{Error, Result},
    grpc::plugin::v1::{handshake_client::HandshakeClient, Empty},
    plugin::{Manifest, PluginInstance},
};
use std::{collections::HashMap, path::PathBuf, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Mutex,
};
use tonic::transport::Channel;
use tracing::{error, info};

/// Manages the lifecycle of all discovered plugins.
///
/// The registry of running plugins is stored in `instances`, which is
/// wrapped in `Arc<Mutex<...>>` to allow for safe concurrent access.
#[derive(Clone, Default, Debug)]
pub struct PluginManager {
    /// The in-memory "registry" of all active plugin instances.
    pub instances: Arc<Mutex<HashMap<String, PluginInstance>>>,
}

impl PluginManager {
    /// Creates a new, empty `PluginManager`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Scans a directory for plugins, spawns them, and performs the handshake.
    ///
    /// # Arguments
    ///
    /// * `plugins_dir` - The path to the directory containing plugin subdirectories.
    pub async fn discover_and_run(&self, plugins_dir: &str) {
        let is_dev_run = match std::env::current_exe() {
            Ok(exe_path) => exe_path.ancestors().any(|p| p.ends_with("target")),
            Err(_) => false,
        };

        let entries = match std::fs::read_dir(plugins_dir) {
            Ok(entries) => entries,
            Err(_) => {
                error!("Plugins directory not found: {}", plugins_dir);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.json");
            if !manifest_path.exists() {
                continue;
            }

            // The rest of the logic is fallible, so we wrap it.
            let manager = self.clone();
            tokio::spawn(async move {
                if let Err(e) = manager.load_plugin(path, is_dev_run).await {
                    error!("Failed to load plugin: {}", e);
                }
            });
        }
    }

    /// Loads, spawns, and handshakes a single plugin.
    async fn load_plugin(&self, path: PathBuf, is_dev_run: bool) -> Result<()> {
        let manifest_str = std::fs::read_to_string(path.join("plugin.json"))?;
        let manifest: Manifest = serde_json::from_str(&manifest_str).map_err(|e| Error::ManifestParse { path: path.clone(), source: e })?;

        info!("Found plugin: {}", manifest.name);

        let executable_from_json = if cfg!(target_os = "windows") {
            &manifest.executable.windows_amd64
        } else {
            &manifest.executable.linux_amd64
        };

        let executable_path = if is_dev_run {
            PathBuf::from(executable_from_json)
        } else {
            path.join(executable_from_json)
        };

        self.spawn_and_handshake(manifest, executable_path).await
    }

    /// Spawns a plugin executable and performs the gRPC handshake protocol.
    async fn spawn_and_handshake(&self, manifest: Manifest, executable_path: PathBuf) -> Result<()> {
        info!("Attempting to spawn plugin from: {:?}", &executable_path);
        let mut child = Command::new(&executable_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::PluginSpawn { name: manifest.name.clone(), path: executable_path, source: e })?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let mut stdout_reader = BufReader::new(stdout).lines();

        // Perform handshake within a timeout.
        let handshake_result = tokio::time::timeout(Duration::from_secs(5), async {
            if let Ok(Some(line)) = stdout_reader.next_line().await {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 5 && parts[4] == "grpc" {
                    let addr = format!("http://{}", parts[3]);
                    if let Ok(channel) = Channel::from_shared(addr).unwrap().connect().await {
                        return Ok(channel); // Handshake seems valid, return channel for verification.
                    }
                }
            }
            // If we reach here, the line was invalid or not received.
            Err(Error::HandshakeInvalid { name: manifest.name.clone(), reason: "Invalid format or no line received".into() })
        }).await;

        match handshake_result {
            Ok(Ok(channel)) => { // Timeout didn't occur and handshake parsing was OK
                // Verify the gRPC connection is actually working.
                let mut client = HandshakeClient::new(channel.clone());
                client.get_plugin_info(Empty {}).await.map_err(|e| Error::GrpcVerification { name: manifest.name.clone(), source: e })?;

                info!("Successfully connected to plugin '{}'", manifest.name);

                let instance = PluginInstance { process: child, manifest: manifest.clone(), grpc_channel: channel };
                self.instances.lock().await.insert(manifest.id.clone(), instance);

                // Spawn tasks to forward plugin logs
                self.spawn_log_forwarders(manifest.name, stdout_reader, stderr);
                Ok(())
            }
            _ => { // Timeout occurred or handshake parsing failed
                error!("Plugin {} did not handshake in time or handshake was invalid.", manifest.name);
                child.kill().await?;
                Err(Error::HandshakeTimeout(manifest.name.clone()))
            }
        }
    }

    /// Spawns background tasks to forward a plugin's stdout and stderr to the tracing system.
    fn spawn_log_forwarders<R: tokio::io::AsyncRead + Unpin + Send + 'static>(&self, name: String, stdout_reader: tokio::io::Lines<BufReader<R>>, stderr: tokio::process::ChildStderr) {
        let out_name = name.clone();
        tokio::spawn(async move {
            let mut lines = stdout_reader;
            while let Ok(Some(line)) = lines.next_line().await {
                info!("[Plugin: {}] {}", out_name, line);
            }
        });

        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                error!("[Plugin: {}] {}", name, line);
            }
        });
    }
}