use clap::{Parser, Subcommand};
use s2n_quic::{client::Connect, Client};
use std::net::ToSocketAddrs;
use std::option::Option;
use s2n_quic::stream::SendStream;
use std::sync::Arc;
use std::{error::Error, path::Path};
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Clone)]
struct ClientConfig {
    source: Vec<ClientSource>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ClientSource {
    r#type: String,
    file_path: String,
    tag: String,
    sink: ClientSink,
}

#[derive(Serialize, Deserialize, Clone)]
struct ClientSink {
    host: String,
    port: u16,
    tls_cert_file: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct MessageFormat {
    timestamp: String,
    tag: String,
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
    }
}

async fn create_server_conn_stream(sink_config: &ClientSink) -> Result<(SendStream), Box<dyn Error>> {
    let server_addr = format!("{}:{}", sink_config.host, sink_config.port)
        .to_socket_addrs()?
        .next()
        .unwrap();
    let client = Client::builder()
        .with_tls(Path::new(&sink_config.tls_cert_file))?
        .with_io("0.0.0.0:0")?
        .start()?;

    
    let connect = Connect::new(server_addr).with_server_name("localhost");
    let mut connection = client.connect(connect).await?;

    // ensure the connection doesn't time out with inactivity
    connection.keep_alive(true)?;
    
    // open a new stream and split the receiving and sending sides
    let stream = connection.open_bidirectional_stream().await?;
    let (_, send_stream) = stream.split();
    Ok(send_stream)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Runner { config_file } => {
            let config: ClientConfig = serde_yaml::from_reader(std::fs::File::open(config_file)?)?;

            // TODO: config.source is array, but for test just use first item
            let first_config = config.source.first().unwrap();
            let file = tokio::fs::File::open(&first_config.file_path).await?;
            let mut reader = tokio::io::BufReader::new(file);
            let mut buffer = String::new();
            let sink_config = Arc::new(first_config.sink.clone());
            let mut send_stream = match create_server_conn_stream(&sink_config).await {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("Error sending message to server: {}", e);
                    return Ok(())
                }
            };
    
            loop {
                // Read new lines from the log file
                let bytes_read = reader.read_line(&mut buffer).await?;
                if bytes_read == 0 {
                    // If no bytes were read, wait for a while before trying again
                    println!("No new line, wait for 5 second");
                    time::sleep(Duration::from_secs(5)).await;
                } else {
                    buffer = buffer.trim().to_string();
                    let escaped_buffer = buffer.replace("\n", "\\n");
                    // Debugging: Print the content of the buffer
                    println!("Buffer content: {}", buffer);
                    let message = MessageFormat {
                        timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis().to_string(),
                        tag: first_config.tag.clone(),
                        message: escaped_buffer
                    };

                    /*
                    The issue you're encountering, where tokio::io::copy does not send the full string of json_string to send_stream, could be due to several reasons related to how asynchronous I/O works in Rust, particularly with the tokio runtime. Here are some potential causes:
                    
                    1. Buffering Behavior:
                    The tokio::io::copy function reads from the source (in this case, json_stream) and writes to the destination (send_stream) in chunks. If the destination stream is not ready to receive data or if there are network issues, it may not send all the data at once.
                    If the send_stream is a network stream, it may have its own internal buffering, which can lead to partial writes if the buffer is full.
                    
                    2. Stream State:
                    If the send_stream is closed or in an error state, it may not accept all the data being sent. Ensure that the connection is still open and valid when you attempt to send data.
                    
                    3. Error Handling:
                    If an error occurs during the tokio::io::copy operation, it may not send all the data. You should check for errors and handle them appropriately. The ? operator will propagate errors, but you may want to log or handle them explicitly to understand what went wrong.
                    
                    4. Data Size:
                    If json_string is particularly large, it may exceed the buffer size of the underlying stream. In such cases, you might need to implement a loop to send the data in smaller chunks manually.
                    
                    Suggested Solution:
                    To ensure that the entire JSON string is sent, you can manually write the data in chunks.
                    */

                    let json_string = serde_json::to_string(&message).unwrap();
                    // println!("Send new line -> {}", json_string);
                    let json_stream = std::io::Cursor::new(json_string);

                    // Manually write the data in chunks
                    let mut temp_buffer = vec![0; 1024]; // Adjust the buffer size as needed
                    let mut cursor = json_stream.clone(); // Clone the cursor to read from it

                    while let Ok(bytes_read) = cursor.read(&mut temp_buffer).await {
                        if bytes_read == 0 {
                            println!("bytes are zero");
                            break; // End of stream
                        }
                        let mut send_buffer = &temp_buffer[..bytes_read];
                        while !send_buffer.is_empty() {

                            // DEBUG send_buffer
                            match std::str::from_utf8(&send_buffer) {
                                Ok(string_value) => {
                                    println!("send_buffer content: {}", string_value);
                                },
                                Err(e) => {
                                    println!("Failed to convert send_buffer to string: {}", e);
                                }
                            }
                            // end DEBUG

                            match send_stream.write(send_buffer).await {
                                Ok(n) => send_buffer = &send_buffer[n..], // Update the remaining buffer
                                Err(e) => {
                                    eprintln!("Error sending data: {}", e);
                                    return Err(e.into());
                                }
                            }
                        }
                    }

                    buffer.clear(); 
                }
            }
        }
    }

    Ok(())
}
