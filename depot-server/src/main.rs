use s2n_quic::stream::BidirectionalStream;
use s2n_quic::Server;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::{error::Error, path::Path};

use depot_common::Config;
use depot_common::MessageQueue;
use tokio::sync::{Mutex, Notify};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let server_addr = format!("{}:{}", &config.server.host, &config.server.port)
        .to_socket_addrs()?
        .next()
        .unwrap();
    let mut server = Server::builder()
        .with_tls((
            Path::new(&config.server.tls.cert_file_path),
            Path::new(&config.server.tls.key_file_path),
        ))?
        .with_io(server_addr)?
        .start()?;
    let queue = Arc::new(Mutex::new(MessageQueue::new()));

    println!("--- Server started in {} ---", server_addr.to_string());
    while let Some(mut connection) = server.accept().await {
        let queue = Arc::clone(&queue);

        tokio::spawn(async move {
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                println!("new connection!!");
                let queue = Arc::clone(&queue);

                tokio::spawn(async move {
                    // First, read the type of connection (writer or reader)
                    if let Ok(Some(connection_type)) = stream.receive().await {
                        match String::from_utf8_lossy(&connection_type.to_vec()).as_ref() {
                            "writer" => {
                                println!("Start writer");
                                // Handle writer connection
                                while let Ok(Some(data)) = stream.receive().await {
                                    println!("Received data from writer: {:?}", data);
                                    let queue_guard = queue.lock().await;
                                    queue_guard.push(data.clone()).await;
                                    println!("Queue contents: {:?}", *queue_guard);
                                }
                            }
                            "reader" => {
                                println!("Start reader");
                                loop {
                                    let queue_guard = queue.lock().await;
                                    if let Some(data) = queue_guard.pop().await {
                                        println!("Sending data to reader: {:?}", data);
                                        if let Err(e) = stream.send(data).await {
                                            eprintln!("Failed to send data to reader: {:?}", e);
                                        }
                                    }
                                }
                            }
                            _ => {
                                eprintln!("Unknown connection type: {:?}", connection_type);
                            }
                        }
                    }
                });
            }
        });
    }

    Ok(())
}

// async fn save_queue_to_file(queue: &MessageQueue) -> std::io::Result<()> {
//     let file = tokio::fs::File::create("sample_queue.txt").await?;
//     let mut writer = tokio::io::BufWriter::new(file);
//     // for item in queue {
//     //     writer.write_all(item.as_bytes()).await?;
//     //     writer.write_all(b"\n").await?;
//     // }

//     Ok(())
// }

async fn handle_read_request(queue: Arc<Mutex<MessageQueue>>, stream: &mut BidirectionalStream) {
    let notify = Arc::new(Notify::new());
    let queue_guard = queue.lock().await;
    while let Some(data) = queue_guard.pop().await {
        // Assuming pop() returns Option<DataType>
        // Send the data back to the client
        if let Err(e) = stream.send(data).await {
            eprintln!("Failed to send data to reader: {:?}", e);
        }
    }

    notify.notify_one();
    // } else {
    //     // Optionally send a message indicating the queue is empty
    //     let empty_message = "Queue is empty".to_string();
    //     let _ = stream.send(empty_message.into()).await;
    // }
}
