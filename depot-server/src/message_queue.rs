use std::collections::VecDeque;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct MessageQueue {
    messages: Arc<Mutex<VecDeque<Bytes>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn push(&self, message: Bytes) {
        let mut queue = self.messages.lock().await;
        queue.push_back(message);
    }

    pub async fn pop(&self) -> Option<Bytes> {
        let mut queue = self.messages.lock().await;
        queue.pop_front()
    }
}
