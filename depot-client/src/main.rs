
use s2n_quic::{client::Connect, Client};
use std::{error::Error, path::Path};
use std::net::{SocketAddr, ToSocketAddrs};

use depot_common::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let server_addr = format!("{}:{}", &config.server.host, &config.server.port).to_socket_addrs()?.next().unwrap();
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
    println!("--- Client started. Write down message and press enter ---");
    let stream = connection.open_bidirectional_stream().await?;
    let (mut receive_stream, mut send_stream) = stream.split();

    // copy responses from the server
    // tokio::spawn(async move {
    //     let mut stdout = tokio::io::stdout();
    //     let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
    // });

    // copy data from stdin and send it to the server
    let mut stdin = tokio::io::stdin();
    tokio::io::copy(&mut stdin, &mut send_stream).await?;

    Ok(())
}
