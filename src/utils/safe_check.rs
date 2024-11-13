use core::panic;

use crate::core::Bot;

pub fn is_connected(bot: &Bot) -> bool {
    match bot.state.lock() {
        Ok(state) => state.is_running,
        Err(_) => panic!("Failed to lock state"),
    }
}
