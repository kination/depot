use bytes::Bytes;
use clap::{Parser, Subcommand};
use s2n_quic::{client::Connect, Client};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::{error::Error, path::Path};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

use depot_common::Config;
use depot_common::MessageQueue;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    File {
        #[clap(long)]
        log_file_path: String,
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
        Commands::File { log_file_path } => {
            println!("--- Read new lines of log file: {} ---", log_file_path);
            let file = tokio::fs::File::open(log_file_path).await?;
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
                    println!("Read line -> {}", buffer);
                    buffer.clear(); // Clear the buffer for the next line
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
