// src/bin/server.rs
use s2n_quic::Server;
use std::{error::Error, path::Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Server file");
    let mut server = Server::builder()
        .with_tls((Path::new("src/cert/cert.pem"), Path::new("src/cert/key.pem")))?
        .with_io("127.0.0.1:4433")?
        .start()?;
    println!("Server started");
    while let Some(mut connection) = server.accept().await {
        // spawn a new task for the connection
        tokio::spawn(async move {
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                // spawn a new task for the stream
                tokio::spawn(async move {
                    // echo any data back to the stream
                    while let Ok(Some(data)) = stream.receive().await {
                        println!("Received data: {:?}", data);
                        stream.send(data).await.expect("stream should be open");
                    }
                });
            }
        });
    }

    Ok(())
}
