use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

type MessageSink = Arc<Mutex<HashMap<String, Box<dyn Fn(String) + Send + 'static>>>>;

#[derive(Default)]
struct Messenger {
    message_sink: MessageSink,
}

impl Messenger {
    pub fn new() -> Self {
        Self {
            message_sink: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn message_sink(&self) -> MessageSink {
        Arc::clone(&self.message_sink)
    }

    pub fn add_listener<F>(&self, message_type: String, callback: F)
    where
        F: Fn(String) + Send + 'static,
    {
        let mut sink = self.message_sink.lock().unwrap();
        sink.insert(message_type, Box::new(callback));
    }

    pub fn send_message(&self, message_type: &str, message_value: &str) {
        let sink = self.message_sink.lock().unwrap();
        if let Some(callback) = sink.get(message_type) {
            callback(message_value.to_string());
        }
    }
}

fn main() {
    let messenger = Messenger::new();

    messenger.add_listener("example".to_string(), |value| {
        println!("Received message: {}", value);
    });

    // 模拟发送消息
    thread::spawn(move || {
        messenger.send_message("example", "Hello, world!");
    })
    .join()
    .unwrap();
}
