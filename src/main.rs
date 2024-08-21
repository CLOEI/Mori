use std::sync::{Arc, Mutex};

use bot::Bot;
use types::elogin_method::ELoginMethod;

mod bot;
mod types;
mod utils;

fn main() {
    let bot = Bot::new(
        "username".to_string(),
        "password".to_string(),
        "recovery_code".to_string(),
        ELoginMethod::LEGACY,
    );
    let bot_arc = Arc::new(Mutex::new(bot));

    bot::logon(&bot_arc);
}
