
use std::sync::Arc;
use s2n_quic::Server;
use s2n_quic::stream::BidirectionalStream;
use std::{error::Error, path::Path};
use std::net::ToSocketAddrs;

use tokio::sync::Mutex;
use depot_common::Config;
use depot_common::MessageQueue;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let server_addr = format!("{}:{}", &config.server.host, &config.server.port).to_socket_addrs()?.next().unwrap();
    let mut server = Server::builder()
        .with_tls((
            Path::new(&config.server.tls.cert_file_path),
            Path::new(&config.server.tls.key_file_path)
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
                    while let Ok(Some(data)) = stream.receive().await {
                        if data == "read" {
                            println!("Read from queue");
                            handle_read_request(queue.clone(), &mut stream).await;
                        } else {
                            println!("Received data: {:?}", data);
                            let queue_guard = queue.lock().await;
                            queue_guard.push(data.clone()).await;
                            println!("Queue contents: {:?}", *queue_guard);
                            drop(queue_guard);
                        }
                        // if let Err(e) = save_queue_to_file(&*queue_guard).await {
                        //     eprintln!("Failed to save queue: {:?}", e);
                        // }
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
    let queue_guard = queue.lock().await;
    if let Some(data) = queue_guard.pop().await { // Assuming pop() returns Option<DataType>
        // Send the data back to the client
        if let Err(e) = stream.send(data).await {
            eprintln!("Failed to send data: {:?}", e);
        }
    } else {
        // Optionally send a message indicating the queue is empty
        let empty_message = "Queue is empty".to_string();
        let _ = stream.send(empty_message.into()).await;
    }
}
