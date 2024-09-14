use std::sync::{Arc, RwLock};
use mlua::Lua;
use mlua::prelude::LuaResult;
use crate::manager::bot_manager::BotManager;
use crate::utils;

pub fn register(lua: &Lua, manager: &Arc<RwLock<BotManager>>) {
    let walk_f = lua.create_function(|_, (x, y, ap): (i32, i32, bool)| -> LuaResult<()> {
        let current_bot = utils::config::get_selected_bot();
        Ok(())
    }).unwrap();
    lua.globals().set("walk", walk_f).unwrap();
}