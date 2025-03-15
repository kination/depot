use rustls::crypto::hash::Hash;
use serde_json::Value;
use rdkafka::ClientConfig;
// use rdkafka::producer::FutureProducer;
use rdkafka::error::KafkaError;
use rdkafka::producer::{FutureRecord, FutureProducer};
use rdkafka::message::{Header, OwnedHeaders};
use std::fs::OpenOptions;
use std::io::Write;
use csv::{Writer, WriterBuilder};
use tokio::sync::oneshot;
use s2n_quic::Server;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::{error::Error, path::Path};
use serde::{Serialize, Deserialize};
use regex::Regex;
use std::collections::HashMap;

use depot_common::Transformer;
use transform_regex::transform::TransformRegexModule;
use transform_demo::transform::TransformDemoModule;


#[derive(Serialize, Deserialize)]
struct ServerConfig {
    inputs: Vec<InputConfig>,
    setting: Settings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct InputConfig {
    tag: String,
    module: String,
    option: Option<HashMap<String, String>>,
    parse: Option<ParseConfig>,
    filter: Option<FilterConfig>,
    produce: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ParseConfig {
    r#type: String,
    exp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FilterConfig {
    rule: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

// TODO: Remove test values, and add key setting
async fn handle_message(message: &str, option: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let option = option.clone();
    match serde_json::to_string(&message) {
        Ok(json_string) => {
            match option["type"].as_str() {
                "kafka" => {
                    let producer: FutureProducer = rdkafka::ClientConfig::new()
                        .set("bootstrap.servers", option["bootstrap_server"].clone()) // Change to your Kafka broker address
                        .create()?;

                    let topic_name = option["topic"].as_str();
                    // println!("Produce message to topic {:?} -> {:?}", topic_name, message);
                    produce_message(&producer, topic_name, message).await;
                }
                "file" => {
                    let filepath = "output.txt";
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(filepath)?;
                    writeln!(file, "{}", message)?;

                }
                /*
                "csv" => {
                    // let key = match transformed_message.get("key") {
                    //     Some(Value::String(s)) => s,
                    //     _ => return Err("JSON must contain a 'key' string field for CSV output".into()),
                    // };
                    let filepath = "output.csv";
                    let mut wtr = WriterBuilder::new()
                        .from_path(filepath)
                        .expect("Failed to create CSV writer");

                    // create/append to file
                    if wtr.get_writer().metadata().len() > 0 {
                         wtr.write_record(&[key])?;
                    } else {
                        wtr.write_record(&[key])?;
                    }

                    wtr.flush()?;
                }
                */
                _ => return Err(format!("Unknown target: {}", option["type"].as_str()).into()),
            }
        }
        Err(e) => {
            println!("The message is not valid JSON: {}", e);
            return Err(e.into());
        }
    }
    Ok(())
}

// TODO: Remove test key/header, and add key setting
async fn produce_message(producer: &FutureProducer, topic_name: &str, message: &str) {
    let payload = serde_json::to_string(message).unwrap();
    let delivery_future = producer.send(
        FutureRecord::to(topic_name).payload(&payload)
                        .key("test-key")
                        .headers(OwnedHeaders::new().insert(Header {
                            key: "header_key",
                            value: Some("header_value"),
                        })),
        std::time::Duration::from_secs(0)
    );
    let _ = delivery_future.await;
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Setup config file path in flexible way
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
                        // Use the "s" flag (DOTALL) to make "." match newlines too
                        let json_regex = Regex::new(r"(?s)\{.*?\}").unwrap();
                        let json_matches: Vec<&str> = json_regex.find_iter(&data_str)
                            .filter_map(|m| Some(m.as_str()))
                            .collect();

                        
                        // Step 3: Deserialize each JSON object
                        for json_str in json_matches {
                            println!("data str -> {:?}", json_str);
                            match serde_json::from_str::<MessageFormat>(json_str) {
                                Ok(message) => {
                                    let flow_option = server_inputs.iter()
                                        .find(|input| input.tag == message.tag) // Find the input with the matching tag
                                        .map(|input| input.clone()) // Get the module name if found
                                        .unwrap();

                                    let transformer = get_transformer(&flow_option.module);

                                    // let option = server_inputs.iter()
                                    //     .find(|input| input.tag == message.tag)
                                    //     .and_then(|input| input.option.clone());
                                    // println!("option: {:?}", option);

                                    // TODO: for test
                                    let target = "kafka";
                                    let run_option = "some_option";
                                    let option = flow_option.option;
                                    match serde_json::to_string(&message) {
                                        Ok(json_string) => {
                                            
                                            let transformed_message = if let Some(transformer) = transformer {
                                                transformer.transform(&json_string, &option).unwrap()
                                            } else {
                                                json_string
                                            };
                                            
                                            let produce_option = Arc::new(flow_option.produce);
                                            // println!("Transformed message: {:?}", transformed_message);
                                            tokio::spawn(async move {
                                                
                                                let option_clone = Arc::clone(&produce_option);
                                                let option_ref = option_clone.as_ref();
                                                let _ = handle_message(&transformed_message, &option_ref).await;
                                            });
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
                    }
                });
            }
        });
    }

    Ok(())
}

fn get_transformer(module_type: &str) -> Option<Box<dyn Transformer>> {
    match module_type {
        "transform-regex" => Some(Box::new(TransformRegexModule)),
        "transform-demo" => Some(Box::new(TransformDemoModule)),
        _ => {
            println!("No matching transformer found for module type: {}", module_type);
            None // Return None if no valid transformer is found
        },
    }
}


