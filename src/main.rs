use std::sync::Arc;
use std::fs::File;
use parquet::file::properties::WriterProperties;
use arrow_array::{Int32Array, ArrayRef,RecordBatch};
use parquet::arrow::arrow_writer::ArrowWriter;
// use tokio::fs::File;

use rdkafka::client::ClientContext;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;

use serde_json::{Result, Value};


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
    let topics = vec!["quickstart"];

    println!("Running kafka consumer...");

    consume_kafka(bootstrap_servers, group_id, &topics).await;
}

// consume kafka message
async fn consume_kafka(bootstrap_servers: &str, group_id: &str, topics: &[&str]) {
    let context = CustomContext;

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", bootstrap_servers)
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Failed getting consumer");

    consumer.subscribe(topics).expect("No subscriptions");

    loop {
        match consumer.recv().await {
            Err(err) => println!("{:?}", err),
            Ok(m) => {
                println!("Received borrowed message");
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        eprintln!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };

                println!("key: {:?}, payload: {:?}, topic: {:?}, partition: {:?}, offset: {:?}", m.key(), payload, m.topic(), m.partition(), m.offset());

                consumer.commit_message(&m, CommitMode::Async).unwrap();

                let ids = Int32Array::from(vec![1, 2, 3, 4]);
                let vals = Int32Array::from(vec![5, 6, 7, 8]);
                let batch = RecordBatch::try_from_iter(vec![
                  ("id", Arc::new(ids) as ArrayRef),
                  ("val", Arc::new(vals) as ArrayRef),
                ]).unwrap();

                let file = File::create("data.parquet").unwrap();
                let props = WriterProperties::builder().build();

                let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props)).unwrap();
                writer.write(&batch).expect("Writing batch");
                writer.close().unwrap();
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

