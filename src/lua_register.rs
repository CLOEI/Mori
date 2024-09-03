use std::sync::Arc;
use mlua::Lua;
use mlua::prelude::LuaResult;
use parking_lot::RwLock;
use crate::manager::Manager;

pub fn register(lua: &Lua, manager: &Arc<RwLock<Manager>>) {
    let walk_f = lua.create_function(|_, (x, y, ap): (i32, i32, bool)| -> LuaResult<()> {
        Ok(())
    }).unwrap();
    lua.globals().set("walk", walk_f).unwrap();
}