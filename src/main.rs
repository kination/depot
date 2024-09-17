use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufRead};

// Simulating a message queue
struct MessageQueue {
    messages: Arc<Mutex<VecDeque<String>>>,
}

impl MessageQueue {
    fn new() -> Self {
        MessageQueue {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn push(&self, message: String) {
        let mut queue = self.messages.lock().unwrap();
        queue.push_back(message);
    }

    fn pop(&self) -> Option<String> {
        let mut queue = self.messages.lock().unwrap();
        queue.pop_front()
    }
}

fn handle_client(stream: TcpStream, queue: Arc<MessageQueue>) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    
    while let Ok(bytes_read) = reader.read_line(&mut line) {
        if bytes_read == 0 {
            break; // Connection closed
        }
        let message = line.trim().to_string();
        queue.push(message);
        println!("Received and queued: {}", line.trim());
        line.clear();
    }
}

fn main() {
    let queue = Arc::new(MessageQueue::new());
    
    // Server (producer) part
    let server_queue = Arc::clone(&queue);
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind");
        println!("Server listening on port 7878");
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client_queue = Arc::clone(&server_queue);
                    thread::spawn(move || {
                        handle_client(stream, client_queue);
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    });

    let consumer_queue = Arc::clone(&queue);
    thread::spawn(move || {
        loop {
            if let Some(message) = consumer_queue.pop() {
                println!("Received: {}", message);
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    });

    // Keep the main thread running
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
