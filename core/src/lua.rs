use std::sync::Arc;

use crate::{types::bot::BotArc, Bot};

pub fn initialize(bot: &Arc<Bot>) {
    let bot_arc = BotArc(bot.clone());
    let lua = &bot.scripting.lua;

    let get_bot = lua
        .create_function(move |_, ()| Ok(bot_arc.clone()))
        .unwrap();
}