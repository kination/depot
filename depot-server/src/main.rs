use rustls::server;
use s2n_quic::stream::BidirectionalStream;
use s2n_quic::Server;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::{error::Error, path::Path};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use regex::Regex;
use tokio::sync::Mutex;


#[derive(Serialize, Deserialize)]
struct ServerConfig {
    inputs: Vec<InputConfig>,
    setting: Settings,
}

#[derive(Serialize, Deserialize)]
struct InputConfig {
    tag: String,
    parse: Option<ParseConfig>,
    filter: Option<FilterConfig>, // Optional since not all inputs have a filter
    produce: Vec<ProduceConfig>,
}

#[derive(Serialize, Deserialize)]
struct ParseConfig {
    r#type: String,
    exp: String,
}

#[derive(Serialize, Deserialize)]
struct FilterConfig {
    rule: String,
}

#[derive(Serialize, Deserialize)]
struct ProduceConfig {
    r#type: String,
    host: Option<String>,
    port: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Settings {
    host: String,
    port: String,
    tls_cert_file: String,
    tls_key_file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MessageFormat {
    timestamp: String,
    tag: String,
    message: String,
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server_config: ServerConfig = serde_yaml::from_reader(
        std::fs::File::open(
            "/Users/kination/workspace/public/depot/configs/sample-server-config.yaml"
        )?
    )?;
    let server_settings = server_config.setting;

    let server_addr = format!("{}:{}", &server_settings.host, &server_settings.port)
        .to_socket_addrs()?
        .next()
        .unwrap();
    let mut server = Server::builder()
        .with_tls((
            Path::new(&server_settings.tls_cert_file),
            Path::new(&server_settings.tls_key_file),
        ))?
        .with_io(server_addr)?
        .start()?;
    // let queue = Arc::new(Mutex::new(MessageQueue::new()));

    println!("--- Server started in {} ---", server_addr.to_string());
    while let Some(mut connection) = server.accept().await {
        // let queue = Arc::clone(&queue);

        tokio::spawn(async move {
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                println!("new connection!!");
                // let queue = Arc::clone(&queue);

                tokio::spawn(async move {
                    while let Ok(Some(data)) = stream.receive().await {
                        // println!("Received data: {:?}", data);
                        if data.is_empty() {
                            println!("No data");
                            continue;
                        }

                        let data_str = String::from_utf8_lossy(&data);
                        // Step 2: Use a regex to find all JSON objects
                        let json_regex = Regex::new(r"\{.*?\}").unwrap(); // Regex to match JSON objects
                        let json_matches: Vec<&str> = json_regex.find_iter(&data_str)
                            .filter_map(|m| Some(m.as_str()))
                            .collect();

                        // Step 3: Deserialize each JSON object
                        for json_str in json_matches {
                            match serde_json::from_str::<MessageFormat>(json_str) {
                                Ok(message) => {
                                    println!("Deserialized message: {:?}", message);
                                    // Process the message as needed
                                },
                                Err(e) => {
                                    println!("Failed to deserialize JSON: {}", e);
                                }
                            }
                        }

                        // let message: MessageFormat = serde_json::from_slice(&data).unwrap();
                        // println!("Deserialized message: {:?}", message);

                        // let queue_guard = queue.lock().await;
                        // queue_guard.push(data.clone()).await;
                        // println!("Queue contents: {:?}", *queue_guard);
                    }
                });
            }
        });
    }

    Ok(())
}

