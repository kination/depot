use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::fs::File;

use rdkafka::client::ClientContext;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;

use serde_json::{Result, Value};
use std::sync::Arc;
use parquet::file::writer::SerializedFileWriter;


/// struct which holds value as json object
struct ReceivedMessage {
    value: serde_json::Value
}

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        println!("Committing offsets: {:?}", result);
    }
}
type LoggingConsumer = StreamConsumer<CustomContext>;

#[tokio::main]
pub async fn main() {
    
    let bootstrap_servers = "localhost:9092";
    let group_id = "group-id";
    let topics = vec!["dummy-topic"];

    consume_kafka(bootstrap_servers, group_id, &topics);
}

// consume kafka message
async fn consume_kafka(bootstrap_servers: &str, group_id: &str, topics: &[&str]) {
    let context = CustomContext;

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", bootstrap_servers)
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(CustomContext)
        .expect("");

    consumer.subscribe(topics).expect("");

    loop {
        match consumer.recv().await {
            Err(err) => eprint!("{:?}", err),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        eprintln!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };

                println!("key: {:?}, payload: {:?}, topic: {:?}, partition: {:?}, offset: {:?}", m.key(), payload, m.topic(), m.partition(), m.offset());

                // if let Some(headers) = m.headers() {
                //     for header in headers.iter() {
                //         info!("  Header {:#?}: {:?}", header.key, header.value);
                //     }
                // }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        }
    }
}

// how can I read schema from kafka message?
async fn schema_from_kafka_msg(msg: &str) {
    let schema = "";


}

/*
write a function which receives kafka messages as string
, and deserialize it to json object
*/
async fn deserialize_kafka_msg(msg: &str) {
    let jsoned: Value = serde_json::from_str(msg).expect("failed to deserialize");

}

