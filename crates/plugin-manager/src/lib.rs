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

                    let executable_path = PathBuf::from(executable_rel_path);
                    self.spawn_and_handshake(manifest, executable_path).await;
                }
            }
        }
    }

    async fn spawn_and_handshake(&self, manifest: Manifest, executable_path: PathBuf) {
        info!("Attempting to spawn plugin from absolute path: {:?}", std::fs::canonicalize(&executable_path));
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

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout).lines();

        if let Ok(Ok(Some(line))) = tokio::time::timeout(Duration::from_secs(5), reader.next_line()).await {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 5 && parts[4] == "grpc" {
                let addr = format!("http://{}", parts[3]);
                match Channel::from_shared(addr).unwrap().connect().await {
                    Ok(channel) => {
                        let mut client = HandshakeClient::new(channel.clone());
                        if client.get_plugin_info(Empty {}).await.is_ok() {
                            info!("Successfully connected to plugin '{}'", manifest.name);
                            let instance = PluginInstance {
                                process: child,
                                manifest: manifest.clone(),
                                grpc_channel: channel,
                            };
                            self.instances.lock().await.insert(manifest.id, instance);
                        }
                    }
                    Err(e) => error!("Failed to connect to gRPC for plugin {}: {}", manifest.name, e),
                }
            }
        } else {
            error!("Plugin {} did not handshake in time.", manifest.name);
            child.kill().await.ok();
        }
    }
}