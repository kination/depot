use std::sync::Arc;
use std::fs::File;
use parquet::file::properties::WriterProperties;
use arrow_array::{Int64Array, StringArray, ArrayRef,RecordBatch};
use arrow::datatypes::{Field,Schema,DataType};
use parquet::arrow::arrow_writer::ArrowWriter;
use arrow_csv::writer::Writer;

use rdkafka::client::ClientContext;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;

use serde::{Deserialize, Serialize};
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


#[derive(Deserialize, Debug, Clone, Serialize)]
struct KafkaConsumerMessage {
    id: i64,
    val: String
}

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
                let cm: KafkaConsumerMessage = serde_json::from_str(payload).unwrap();
                let rb: RecordBatch = get_record_batch(cm);

                write_as_parquet(rb.clone(), "test.parquet");
                write_as_csv(rb.clone(), "test.csv");
            }
        }
    }
}

fn get_record_batch(msg: KafkaConsumerMessage) -> RecordBatch {
    let ids = Int64Array::from(vec![msg.id]);
    let vals = StringArray::from(vec![msg.val]);

    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("val", DataType::Utf8, false)
    ]);

    RecordBatch::try_new(Arc::new(schema), vec![
        Arc::new(ids) as ArrayRef,
        Arc::new(vals) as ArrayRef,
    ]).unwrap()
}

fn write_as_csv(batch: RecordBatch, file_name: &str) {
    let file = File::create(file_name).unwrap();

    let mut writer = Writer::new(file);
    let batches = vec![&batch, &batch];

    for batch in batches {
        writer.write(batch).unwrap();
    }
}

fn write_as_parquet(batch: RecordBatch, file_name: &str) {
    let file = File::create(file_name).unwrap();

    let mut writer = ArrowWriter::try_new(
        file, 
        batch.schema(),
        Some(WriterProperties::builder().build())
    ).unwrap();

    writer.write(&batch).unwrap();
    writer.close().unwrap();
}
