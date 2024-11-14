use clap::{Parser, Subcommand};
use s2n_quic::{client::Connect, Client};
use std::net::{SocketAddr, ToSocketAddrs};
use s2n_quic::stream::BidirectionalStream;
use s2n_quic::Server;
use std::sync::Arc;
use std::collections::HashMap;
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
    schema: HashMap<String, String>,
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

async fn send_message_to_server(json_message: String) -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let server_addr = format!("{}:{}", &config.server.host, &config.server.port)
        .to_socket_addrs()?
        .next()
        .unwrap();
    let client = Client::builder()
        .with_tls(Path::new(&config.server.tls.cert_file_path))?
        .with_io("0.0.0.0:0")?
        .start()?;

    
    let connect = Connect::new(server_addr).with_server_name("localhost");
    let mut connection = client.connect(connect).await?;

    // ensure the connection doesn't time out with inactivity
    connection.keep_alive(true)?;
    
    // open a new stream and split the receiving and sending sides
    let stream = connection.open_bidirectional_stream().await?;
    let (mut receive_stream, mut send_stream) = stream.split();

    // spawn a task that copies responses from the server to stdout
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
    });

    // copy data from stdin and send it to the server
    let mut json_stream = std::io::Cursor::new(json_message);
    tokio::io::copy(&mut json_stream, &mut send_stream).await?;

    Ok(())
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
                    let re = Regex::new(
                        &config.regex
                    ).unwrap();
                    
                    if let Some(captures) = re.captures(&buffer) {
                        let mut json_message = serde_json::json!({});
                        // Iterate over the keys in the schema HashMap
                        for (key, field_type) in &config.schema {
                            if let Some(value) = captures.name(key) {
                                json_message[key] = serde_json::json!(value.as_str());
                            }
                        }

                        println!("JSON Output -> {}", json_message);
                        let json_string = serde_json::to_string(&json_message).unwrap();
                        tokio::spawn(
                            async move { 
                                send_message_to_server(json_string).await;
                            }
                        );
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
