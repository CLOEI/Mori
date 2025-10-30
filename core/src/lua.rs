use std::sync::Arc;

use crate::{Bot, types::bot::BotArc};

pub fn initialize(bot: &Arc<Bot>) {
    let bot_arc = BotArc(bot.clone());
    let lua = &bot.scripting.lua;

    let get_bot = lua
        .create_function(move |_, ()| Ok(bot_arc.clone()))
        .unwrap();
    let sleep = lua
        .create_function(move |_, duration: u64| {
            std::thread::sleep(std::time::Duration::from_millis(duration));
            Ok(())
        })
        .unwrap();

    lua.globals().set("getBot", get_bot).unwrap();
    lua.globals().set("sleep", sleep).unwrap();
}
