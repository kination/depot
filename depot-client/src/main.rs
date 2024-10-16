use bytes::Bytes;
use clap::{Parser, Subcommand};
use s2n_quic::{client::Connect, Client};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::{error::Error, path::Path};
use tokio::io::AsyncWriteExt;
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

            // Open a new bidirectional stream
            let mut stream = connection.open_bidirectional_stream().await.unwrap();
            stream.write_all("reader".as_bytes()).await.unwrap();
            let (mut receive_stream, mut send_stream) = stream.split(); // Split the stream here

            println!("--- Read client started. ---");
            // Send the read command
            // send_stream.send(Bytes::from("read")).await?;

            // Receive the response from the server
            loop {
                // Attempt to receive data
                if let Ok(Some(data)) = receive_stream.receive().await {
                    println!("Received data: {:?}", data);
                }

                // Sleep for a specified duration before the next iteration
                time::sleep(Duration::from_secs(1)).await; // Adjust the duration as needed
            }
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

            // open a new bidirection stream
            println!("--- Write client started. Write down message and press enter ---");
            let mut stream = connection.open_bidirectional_stream().await.unwrap();
            stream.write_all("writer".as_bytes()).await.unwrap();
            let (mut receive_stream, mut send_stream) = stream.split();

            // copy responses from the server
            // tokio::spawn(async move {
            //     let mut stdout = tokio::io::stdout();
            //     let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
            // });

            // copy data from stdin and send it to the server
            let mut stdin = tokio::io::stdin();
            tokio::io::copy(&mut stdin, &mut send_stream).await?;
        }
    }

    Ok(())
}
