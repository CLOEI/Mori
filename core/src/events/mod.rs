mod broadcaster;
mod types;

pub use broadcaster::EventBroadcaster;
pub use types::{BotEvent, EventType, LogLevel};

use std::sync::mpsc;

pub fn create_event_channel() -> (EventBroadcaster, mpsc::Receiver<BotEvent>) {
    let (sender, receiver) = mpsc::channel();
    (EventBroadcaster::new(sender), receiver)
}
