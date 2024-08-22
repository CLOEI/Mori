use std::{sync::Arc, thread, time::Duration};

use bot::Bot;
use types::elogin_method::ELoginMethod;

mod bot;
mod types;
mod utils;

fn main() {
    let bot = Arc::new(Bot::new(
        "".to_string(),
        "".to_string(),
        "recovery_code".to_string(),
        ELoginMethod::LEGACY,
    ));

    let bot_clone = bot.clone();
    thread::spawn(move || bot::logon(&bot_clone));

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
