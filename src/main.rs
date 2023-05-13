pub mod config;
pub mod schema;
pub mod sink;

use crate::schema::{get_record_batch, KafkaConsumerMessage};
use crate::sink::{write_as_csv, write_as_parquet};

use std::time::Duration;
use tokio::{task, time};
use tokio::time::{sleep};

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use async_channel::{unbounded, Sender};
use futures_lite::future;

/// struct which holds value as json object
struct ReceivedMessage {
    value: serde_json::Value,
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
    
    let mut is_tick_open = false;
    let mut curr_time = time::Instant::now();
    let next_tick = time::Instant::now() + Duration::from_secs(30);
    let (q_send, q_recv) = unbounded::<Vec<u8>>();
    
    task::spawn(async move {
        // sleep(Duration::from_secs(10)).await;
        let mut interval = time::interval(Duration::from_secs(1));
        println!("in loop");
        
        loop { 
            interval.tick().await;

            if curr_time.elapsed() > Duration::from_secs(15) {
                println!("--------- 15 second passed : {:?}", q_recv.len());
                let mut list = Vec::new();
                while !q_recv.is_empty() {
                    let received = future::block_on(q_recv.recv()).unwrap();
                    let cm: KafkaConsumerMessage = serde_json::from_slice(&received).unwrap();
                    list.push(cm);
                    println!("received : {:?}", String::from_utf8(received));
                }
                let rb = get_record_batch(list);

                println!("Write data from kafka to file");
                // TODO: using 'clone' here is not ideal
                // write_as_parquet(rb.clone(), "test.parquet");
                write_as_csv(rb.clone(), "test.csv");
                
                curr_time = time::Instant::now();
            }
        }
    });

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

                println!(
                    "key: {:?}, payload: {:?}, topic: {:?}, partition: {:?}, offset: {:?}",
                    m.key(),
                    payload,
                    m.topic(),
                    m.partition(),
                    m.offset()
                );

                consumer.commit_message(&m, CommitMode::Async).unwrap();
                // let cm: KafkaConsumerMessage = serde_json::from_str(payload).unwrap();
                q_send.send(payload.as_bytes().to_vec()).await;
                println!("Message has been received");
            }
        }

        println!("out of matches");
    }
}


#[cfg(test)]
mod tests {
    use tokio::*;
    // use std::time::Duration;
    use tokio::{task, time};
    use std::{
        thread,
        time::{Duration, Instant},
    };

    use crate::schema::KafkaConsumerMessage; // 1.3.0

    #[tokio::test]
    async fn it_works() {
        
        // fn test_main() {
            let scheduler = thread::spawn(|| {
                let wait_time = Duration::from_secs(10);
        
                // Make this an infinite loop
                // Or some control path to exit the loop
                for _ in 0..5 {
                    let start = Instant::now();
                    println!("Scheduler starting at {:?}", start);
        
                    let thread_a = thread::spawn(a);
                    let thread_b = thread::spawn(b);
        
                    thread_a.join().expect("Thread A panicked");
                    thread_b.join().expect("Thread B panicked");
        
                    let runtime = start.elapsed();
        
                    if let Some(remaining) = wait_time.checked_sub(runtime) {
                        println!(
                            "schedule slice has time left over; sleeping for {:?}",
                            remaining
                        );
                        thread::sleep(remaining);
                    }
                }
            });
        
            scheduler.join().expect("Scheduler panicked");
        // }
        
        fn a() {
            println!("a");
            thread::sleep(Duration::from_millis(100))
        }
        fn b() {
            println!("b");
            thread::sleep(Duration::from_millis(200))
        }
    }
}
