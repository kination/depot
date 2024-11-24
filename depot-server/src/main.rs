use rustls::server;
use s2n_quic::Server;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::{error::Error, path::Path};
use serde::{Serialize, Deserialize};
use regex::Regex;
use tokio::sync::Mutex;

use depot_common::Transformer;
use transform_demo::transform::TransformDemoModule;
use transform_demo_second::transform_second::TransformDemoModuleSecond;



#[derive(Serialize, Deserialize)]
struct ServerConfig {
    inputs: Vec<InputConfig>,
    setting: Settings,
}

#[derive(Serialize, Deserialize, Clone)]
struct InputConfig {
    tag: String,
    module: String,
    parse: Option<ParseConfig>,
    filter: Option<FilterConfig>,
    produce: Vec<ProduceConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ParseConfig {
    r#type: String,
    exp: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct FilterConfig {
    rule: String,
}

#[derive(Serialize, Deserialize, Clone)]
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
    let server_inputs = server_config.inputs;

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

    println!("--- Server started in {} ---", server_addr.to_string());
    let server_inputs = Arc::new(server_inputs); 
    while let Some(mut connection) = server.accept().await {
        // let queue = Arc::clone(&queue);
        let server_inputs = Arc::clone(&server_inputs);
        tokio::spawn(async move {
            
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                println!("new connection!!");
                let server_inputs = Arc::clone(&server_inputs);
                tokio::spawn(async move {
                    while let Ok(Some(data)) = stream.receive().await {
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
                                    let module_type = server_inputs.iter()
                                        .find(|input| input.tag == message.tag) // Find the input with the matching tag
                                        .map(|input| input.module.clone()) // Get the module name if found
                                        .unwrap_or_else(|| {
                                            println!("No matching module found for tag: {}", message.tag);
                                            String::new()
                                        });

                                    let transformer: Box<dyn Transformer> = match module_type.as_str() {
                                        "transform-demo" => Box::new(TransformDemoModule),
                                        "transform-demo-second" => Box::new(TransformDemoModuleSecond),
                                        _ => {
                                            println!("No matching transformer found for module type: {}", module_type);
                                            return;
                                        },
                                    };    
                                    
                                    match serde_json::to_string(&message) {
                                        Ok(json_string) => {
                                            println!("The message is valid JSON: {}", json_string);
                                            let transformed_message = transformer.transform(&json_string);
                                            println!("Transformed message: {}", transformed_message);
                                        },
                                        Err(e) => {
                                            println!("The message is not valid JSON: {}", e);
                                        }
                                    }
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

