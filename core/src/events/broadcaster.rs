use std::sync::mpsc::Sender;
use super::types::BotEvent;

#[derive(Debug, Clone)]
pub struct EventBroadcaster {
    sender: Sender<BotEvent>,
}

impl EventBroadcaster {
    pub fn new(sender: Sender<BotEvent>) -> Self {
        Self { sender }
    }

    pub fn emit(&self, event: BotEvent) {
        let _ = self.sender.send(event);
    }
}
