use std::collections::VecDeque;

use valence::protocol::Username;

pub struct MessageQueue {
    messages: VecDeque<String>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    pub fn queue_chat(&mut self, username: Username<&str>, message: String, timestamp: u64) {
        let message = format!("[{}] {}: {}", timestamp, username, message);

        self.messages.push_back(message);
    }

    pub fn pop_front(&mut self) -> Option<String> {
        self.messages.pop_front()
    }
}
