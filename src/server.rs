// src/bin/server.rs
mod message_queue;

use std::sync::Arc;
use s2n_quic::Server;
use std::{error::Error, path::Path};
use tokio::sync::Mutex;

use crate::message_queue::MessageQueue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let queue = Arc::new(MessageQueue::new());

    let mut server = Server::builder()
        .with_tls((Path::new("src/cert/cert.pem"), Path::new("src/cert/key.pem")))?
        .with_io("127.0.0.1:4433")?
        .start()?;

    println!("--- Server started ---");
    while let Some(mut connection) = server.accept().await {
        let queue = Arc::new(Mutex::new(MessageQueue::new()));
        // spawn a new task for the connection
        tokio::spawn(async move {
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                let queue = Arc::clone(&queue);
                // spawn a new task for the stream
                tokio::spawn(async move {
                    // echo any data back to the stream
                    while let Ok(Some(data)) = stream.receive().await {
                        println!("Received data: {:?}", data);
                        let queue_guard = queue.lock().await;
                        queue_guard.push(data.clone()).await;

                        // Part for debugging....
                        println!("Queue contents: {:?}", *queue_guard);
                        drop(queue_guard);
                        // stream.send(data).await.expect("stream should be open");
                    }
                });
            }
        });
    }

    Ok(())
}
