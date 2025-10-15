use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Stdio};
use tokio::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::{error, info, warn};

pub mod proto {
    pub mod plugin {
        pub mod v1 {
            tonic::include_proto!("plugin.v1");
        }
    }
}

use proto::plugin::v1::handshake_client::HandshakeClient;
use proto::plugin::v1::Empty;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ExecutableConfig {
    pub linux_amd64: String,
    pub windows_amd64: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct FrontendConfig {
    pub entry: String,
    pub route: String,
    #[serde(rename = "sidebarLabel")]
    pub sidebar_label: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub executable: ExecutableConfig,
    pub frontend: FrontendConfig,
}

pub struct PluginInstance {
    pub process: Child,
    pub manifest: Manifest,
    pub grpc_channel: Channel,
}

#[derive(Clone, Default)]
pub struct PluginManager {
    pub instances: Arc<Mutex<HashMap<String, PluginInstance>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn discover_and_run(&self, plugins_dir: &str) {
        let Ok(entries) = std::fs::read_dir(plugins_dir) else {
            warn!("Plugins directory not found: {}", plugins_dir);
            return;
        };

        let exe_path = std::env::current_exe().unwrap();
        let is_dev_run = exe_path.ancestors().any(|p| p.ends_with("target"));

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.json");
                if manifest_path.exists() {
                    let manifest = match serde_json::from_str::<Manifest>(&std::fs::read_to_string(&manifest_path).unwrap()) {
                        Ok(m) => m,
                        Err(e) => {
                            error!("Failed to parse manifest for {:?}: {}", path, e);
                            continue;
                        }
                    };

                    info!("Found plugin: {}", manifest.name);
                    let executable_rel_path = if cfg!(target_os = "windows") {
                        &manifest.executable.windows_amd64
                    } else if cfg!(target_os = "linux") {
                        &manifest.executable.linux_amd64
                    } else {
                        // Add more OS targets as needed, e.g., macos
                        error!("Unsupported OS for plugin {}", manifest.name);
                        continue;
                    };

                    let executable_path = if is_dev_run {
                        // In dev mode, the path from JSON is relative to the workspace root.
                        PathBuf::from(executable_rel_path)
                    } else {
                        // In prod mode, the path from JSON is relative to its own plugin folder.
                        path.join(executable_rel_path)
                    };
                    self.spawn_and_handshake(manifest, executable_path).await;
                }
            }
        }
    }

    async fn spawn_and_handshake(&self, manifest: Manifest, executable_path: PathBuf) {
        info!("Attempting to spawn plugin from: {:?}", &executable_path);
        let mut child = match Command::new(executable_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                error!("Failed to spawn plugin {}: {}", manifest.name, e);
                return;
            }
        };

        // Take ownership of the output handles.
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let mut stdout_reader = BufReader::new(stdout).lines();

        // --- RESTRUCTURED HANDSHAKE AND OWNERSHIP LOGIC ---

        // Perform the handshake and get the gRPC channel if successful.
        let handshake_result = tokio::time::timeout(Duration::from_secs(5), async {
            if let Ok(Some(line)) = stdout_reader.next_line().await {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 5 && parts[4] == "grpc" {
                    let addr = format!("http://{}", parts[3]);
                    if let Ok(channel) = Channel::from_shared(addr).unwrap().connect().await {
                        let mut client = HandshakeClient::new(channel.clone());
                        if client.get_plugin_info(Empty {}).await.is_ok() {
                            return Some(channel); // Handshake success! Return the channel.
                        }
                    }
                }
            }
            None // Handshake failed.
        }).await;

        match handshake_result {
            // Handshake succeeded within the timeout.
            Ok(Some(channel)) => {
                info!("Successfully connected to plugin '{}'", manifest.name);

                // Now we can safely move `child` into the instance.
                let instance = PluginInstance {
                    process: child,
                    manifest: manifest.clone(),
                    grpc_channel: channel,
                };
                self.instances.lock().await.insert(manifest.id, instance);

                // Spawn logging tasks for the remaining output.
                let plugin_name_stdout = manifest.name.clone();
                tokio::spawn(async move {
                    let mut remaining_lines = stdout_reader;
                    while let Ok(Some(line)) = remaining_lines.next_line().await {
                        info!("[Plugin: {}] {}", plugin_name_stdout, line);
                    }
                });

                let plugin_name_stderr = manifest.name.clone();
                tokio::spawn(async move {
                    let mut reader = BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        error!("[Plugin: {}] {}", plugin_name_stderr, line);
                    }
                });
            }
            // Handshake failed or timed out.
            _ => {
                error!("Plugin {} did not handshake in time or handshake was invalid.", manifest.name);
                // `child` is still owned here, so we can safely kill it.
                child.kill().await.ok();
            }
        }
    }
}