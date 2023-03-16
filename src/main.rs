pub mod sink;
pub mod schema;

use crate::schema::{KafkaConsumerMessage, get_record_batch};
use crate::sink::{write_as_csv, write_as_parquet};

use arrow_array::{RecordBatch};

use rdkafka::client::ClientContext;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;


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
                let cm: KafkaConsumerMessage = serde_json::from_str(payload).unwrap();
                let rb: RecordBatch = get_record_batch(cm);

                // TODO: using 'clone' here is not ideal
                write_as_parquet(rb.clone(), "test.parquet");
                write_as_csv(rb.clone(), "test.csv");
            }
        }
    }
}
