use signals2::Signal;

struct Messenger {
    message_sink: Signal<(String, String)>,
}

impl Messenger {
    fn new() -> Self {
        Messenger {
            message_sink: Signal::new(),
        }
    }

    fn message_sink(&mut self) -> &mut Signal<(String, String)> {
        &mut self.message_sink
    }
}
