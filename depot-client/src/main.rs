use clap::{Parser, Subcommand};
use s2n_quic::{client::Connect, Client};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::{error::Error, path::Path};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use regex::Regex;
use serde::Deserialize;

use depot_common::Config;
use depot_common::MessageQueue;


#[derive(Debug, Deserialize)]
struct RunnerConfig {
    r#type: String,
    file_path: String,
    regex: String,
    output: String,
    schema: RunnerSchema,
}

#[derive(Debug, Deserialize)]
struct RunnerSchema {
    timestamp: String,
    process: String,
    message: String,
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Runner {
        #[clap(long)]
        config_file: String
    },
    Write {
        #[clap(long)]
        host: Option<String>,

        #[clap(long)]
        port: Option<String>,

        #[clap(long)]
        tls_cert_file_path: Option<String>,
    },
    Read {
        #[clap(long)]
        host: Option<String>,

        #[clap(long)]
        port: Option<String>,

        #[clap(long)]
        tls_cert_file_path: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Runner { config_file } => {
            let config: RunnerConfig = serde_yaml::from_reader(std::fs::File::open(config_file)?)?; // Add this line to read the config

            println!("--- Read new lines of log file: {} ---", config.file_path);
            let file = tokio::fs::File::open(config.file_path).await?;
            let mut reader = tokio::io::BufReader::new(file);
            let mut buffer = String::new();
    
            loop {
                // Read new lines from the log file
                let bytes_read = reader.read_line(&mut buffer).await?;
                if bytes_read == 0 {
                    // If no bytes were read, wait for a while before trying again
                    println!("No new line, wait for 10 second");
                    time::sleep(Duration::from_secs(10)).await;
                } else {
                    // Print the new line read from the log file
                    let re = Regex::new(r"(?s)^(?P<timestamp>\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(?P<host>[\w-]+(?:\s+[\w-]+)*)\s+(?P<process>[\w-]+\[\d+\])?:\s+(?P<message>.+)$").unwrap();

                    // Apply the regex to the buffer
                    if let Some(captures) = re.captures(&buffer) {
                        let json_message = serde_json::json!({
                            "timestamp": &captures["timestamp"],
                            "host": &captures["host"],
                            "process": &captures["process"],
                            "message": &captures["message"]
                        });

                        println!("JSON Output -> {}", json_message);
                    } else {
                        println!("No message captured by regex")
                    }

                    buffer.clear(); 
                }
            }
        },
        Commands::Read {
            host,
            port,
            tls_cert_file_path,
        } => {
            let config = Config::new();
            let server_addr = format!("{}:{}", &config.server.host, &config.server.port)
                .to_socket_addrs()?
                .next()
                .unwrap();
            let client = Client::builder()
                .with_tls(Path::new(&config.server.tls.cert_file_path))?
                .with_io("0.0.0.0:0")?
                .start()?;

            let addr: SocketAddr = server_addr;
            let connect = Connect::new(addr).with_server_name("localhost");
            let mut connection = client.connect(connect).await?;
        }
        Commands::Write {
            host,
            port,
            tls_cert_file_path,
        } => {
            let config = Config::new();
            let server_addr = format!("{}:{}", &config.server.host, &config.server.port)
                .to_socket_addrs()?
                .next()
                .unwrap();
            let client = Client::builder()
                .with_tls(Path::new(&config.server.tls.cert_file_path))?
                .with_io("0.0.0.0:0")?
                .start()?;

            let addr: SocketAddr = server_addr;
            let connect = Connect::new(addr).with_server_name("localhost");
            let mut connection = client.connect(connect).await?;

            // ensure the connection doesn't time out with inactivity
            connection.keep_alive(true)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let regex = Regex::new(r"(?m)^(?P<timestamp>\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(?P<host>[\w-]+(?:\s+[\w-]+)*)\s+(?P<process>[\w-]+\[\d+\])?:\s+(?P<message>.+)$").unwrap();
        let string = "Nov  3 16:36:42 kinations-MacBook-Air login[2735]: DEAD_PROCESS: 2735 ttys003
        
        ";
  
        let result = regex.captures_iter(string);
        for mat in result {
            println!("{:?}", mat);
        }
    }
}
