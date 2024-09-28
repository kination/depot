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

// pub struct ClientConfig {
//     pub server_host: String,
//     pub server_port: u16,
//     pub tls: TlsConfig
// }

impl Config {
    pub fn new() -> Self {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // println!("Project root: {}", project_root.display());
        let config_file_path = project_root.join("config.yaml");
        let config_file_content = fs::File::open(config_file_path).expect("Failed to read config file");
        let config: Config = serde_yaml::from_reader(config_file_content).expect("Failed to parse config file");
        config
    }
}
