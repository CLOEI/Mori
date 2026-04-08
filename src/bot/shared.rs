use crate::protocol::packet::GameUpdatePacket;
use crate::protocol::variant::VariantList;
use std::net::SocketAddr;
use std::sync::Mutex;

use super::core::Bot;

#[derive(Clone, Debug)]
pub struct Socks5Config {
    pub proxy_addr: SocketAddr,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Socks5Config {
    pub fn to_url(&self) -> String {
        match (&self.username, &self.password) {
            (Some(u), Some(p)) => format!("socks5://{}:{}@{}", u, p, self.proxy_addr),
            _ => format!("socks5://{}", self.proxy_addr),
        }
    }
}

/// Raw event pushed to `Bot::event_queue` by packet handlers.
/// Drained by Lua's `listenEvents` loop to fire registered callbacks.
pub enum BotEventRaw {
    VariantList { vl: VariantList, net_id: u32 },
    GameUpdate { pkt: GameUpdatePacket },
    GameMessage { text: String },
}

/// Callback invoked on the next `OnDialogRequest`, then cleared.
type DialogCallback = Box<dyn FnOnce(&mut Bot) + Send>;

pub struct TemporaryData {
    pub dialog_callback: Mutex<Option<DialogCallback>>,
}

impl Default for TemporaryData {
    fn default() -> Self {
        Self {
            dialog_callback: Mutex::new(None),
        }
    }
}
