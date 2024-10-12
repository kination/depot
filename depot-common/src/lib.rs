use std::collections::VecDeque;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls: TlsConfig
}

#[derive(Debug, Deserialize)]
pub struct TlsConfig {
    pub cert_file_path: String,
    pub key_file_path: String,
}

impl Config {
    pub fn new() -> Self {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config_file_path = project_root.parent().unwrap().join("config.yaml");
        let config_file_content = fs::File::open(config_file_path).expect("Failed to read config file");
        let config: Config = serde_yaml::from_reader(config_file_content).expect("Failed to parse yaml config file");
        config
    }
}


#[derive(Debug)]
pub struct MessageQueue {
    messages: Arc<Mutex<VecDeque<Bytes>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn push(&self, message: Bytes) {
        let mut queue = self.messages.lock().await;
        queue.push_back(message);
    }

    pub async fn pop(&self) -> Option<Bytes> {
        let mut queue = self.messages.lock().await;
        queue.pop_front()
    }
}

