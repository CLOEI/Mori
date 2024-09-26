use std::sync::mpsc::Sender;
use paris::{error, info, warn};

pub fn info(message: &str, sender: &Sender<String>) {
    info!("{}", message);
    sender.send(format!("info|{}", message)).unwrap();
}

pub fn warn(message: &str, sender: &Sender<String>) {
    warn!("{}", message);
    sender.send(format!("warn|{}", message)).unwrap();
}

pub fn error(message: &str, sender: &Sender<String>) {
    error!("{}", message);
    sender.send(format!("error|{}", message)).unwrap();
}