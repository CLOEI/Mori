use std::sync::Arc;
use mlua::Lua;
use mlua::prelude::LuaResult;
use parking_lot::RwLock;
use crate::manager::Manager;
use crate::utils;

pub fn register(lua: &Lua, manager: &Arc<RwLock<Manager>>) {
    let walk_f = lua.create_function(|_, (x, y, ap): (i32, i32, bool)| -> LuaResult<()> {
        let current_bot = utils::config::get_selected_bot();
        Ok(())
    }).unwrap();
    lua.globals().set("walk", walk_f).unwrap();
}