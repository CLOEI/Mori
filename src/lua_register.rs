use std::sync::{Arc};
use std::thread;
use std::time::Duration;
use mlua::prelude::*;
use crate::bot::{disconnect, drop_item, find_path, is_inworld, leave, place, punch, talk, trash_item, walk, warp, wear, Bot};

fn register_function<'lua, F, A, R>(lua: &'lua Lua, name: &str, bot: &Arc<Bot>, func: F) -> LuaResult<()>
where
    F: 'static + Send + Fn(&Arc<Bot>, A) -> R,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
{
    let bot_clone = bot.clone();
    let lua_func = lua.create_function(move |_, args: A| {
        func(&bot_clone, args);
        Ok(())
    })?;
    lua.globals().set(name, lua_func)
}

pub fn register(lua: &Lua, bot: &Arc<Bot>) {
    register_function(lua, "walk", bot, |bot, (x, y, ap): (i32, i32, bool)| {
        walk(bot, x, y, ap);
    }).unwrap();
    register_function(lua, "leave", bot, |bot, ()| {
        leave(bot);
    }).unwrap();
    register_function(lua, "disconnect", bot, |bot, ()| {
        disconnect(bot);
    }).unwrap();
    register_function(lua, "is_in_world", bot, |bot, ()| {
        is_inworld(bot);
    }).unwrap();
    register_function(lua, "place", bot, |bot, (offset_x, offset_y, item_id): (i32, i32, u32)| {
        place(bot, offset_x, offset_y, item_id);
    }).unwrap();
    register_function(lua, "punch", bot, |bot, (offset_x, offset_y): (i32, i32)| {
        punch(bot, offset_x, offset_y);
    }).unwrap();
    register_function(lua, "wear", bot, |bot, item_id: u32| {
        wear(bot, item_id);
    }).unwrap();
    register_function(lua, "warp", bot, |bot, world_name: String| {
        warp(bot, world_name);
    }).unwrap();
    register_function(lua, "talk", bot, |bot, message: String| {
        talk(bot, message);
    }).unwrap();
    register_function(lua, "find_path", bot, |bot, (x, y): (u32, u32)| {
        find_path(bot, x, y);
    }).unwrap();
    register_function(lua, "drop", bot, |bot, (item_id, amount): (u32, u32)| {
        drop_item(bot, item_id, amount);
    }).unwrap();
    register_function(lua, "trash", bot, |bot, (item_id, amount): (u32, u32)| {
        trash_item(bot, item_id, amount);
    }).unwrap();
    register_function(lua, "sleep", bot, |_, ms: u64| {
        thread::sleep(Duration::from_millis(ms));
    }).unwrap();
}