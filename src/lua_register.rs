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

    let bot_clone = bot.clone();
    let get_world = lua.create_function(move |lua, ()| -> LuaResult<LuaValue> {
        let world_data = lua.create_table()?;
        let world = bot_clone.world.read().unwrap();

        world_data.set("name", world.name.clone())?;
        world_data.set("width", world.width)?;
        world_data.set("height", world.height)?;
        world_data.set("tiles_count", world.tile_count)?;

        let tiles_table = lua.create_table()?;
        for (i, tile) in world.tiles.clone().into_iter().enumerate() {
            let tile_clone = tile.clone();
            let bot_clone = bot_clone.clone();
            let tile_table = lua.create_table()?;
            let tile_harvestable_func = lua.create_function(move |_, ()| -> LuaResult<bool> {
                let world = bot_clone.world.read().unwrap();
                let harvestable = world.is_tile_harvestable(&tile_clone);
                Ok(harvestable)
            })?;
            tile_table.set("fg", tile.foreground_item_id)?;
            tile_table.set("bg", tile.background_item_id)?;
            tile_table.set("pbi", tile.parent_block_index)?;
            tile_table.set("flags", tile.flags)?;
            tile_table.set("x", tile.x)?;
            tile_table.set("y", tile.y)?;
            tile_table.set("harvestable", tile_harvestable_func)?;
            tiles_table.set(i + 1, tile_table)?;
        }
        world_data.set("tiles", tiles_table)?;

        let dropped_table = lua.create_table()?;
        let dropped_items_table = lua.create_table()?;
        dropped_table.set("count", world.dropped.items_count)?;
        dropped_table.set("last_dropped_item_uid", world.dropped.last_dropped_item_uid)?;
        for (i, dropped) in world.dropped.items.clone().into_iter().enumerate() {
            let dropped_table = lua.create_table()?;
            dropped_table.set("id", dropped.id)?;
            dropped_table.set("x", dropped.x)?;
            dropped_table.set("y", dropped.y)?;
            dropped_table.set("count", dropped.count)?;
            dropped_table.set("flags", dropped.flags)?;
            dropped_table.set("uid", dropped.uid)?;
            dropped_items_table.set(i + 1, dropped_table)?;
        }
        dropped_table.set("items", dropped_items_table)?;
        world_data.set("dropped", dropped_table)?;

        world_data.set("base_weather", world.base_weather)?;
        world_data.set("current_weather", world.current_weather)?;
        world_data.set("is_error", world.is_error)?;

        Ok(LuaValue::Table(world_data))
    }).unwrap();
    lua.globals().set("get_world", get_world).unwrap();

    let bot_clone = bot.clone();
    let get_local = lua.create_function(move |lua, ()| -> LuaResult<LuaValue> {
        let local_data = lua.create_table()?;
        let position = bot_clone.position.read().unwrap();
        let state = bot_clone.state.read().unwrap();

        local_data.set("x", position.x)?;
        local_data.set("y", position.y)?;
        local_data.set("net_id", state.net_id)?;
        local_data.set("level", state.level)?;
        local_data.set("gems", state.gems)?;

        Ok(LuaValue::Table(local_data))
    }).unwrap();
    lua.globals().set("get_local", get_local).unwrap();

    let bot_clone = bot.clone();
    let get_inventory = lua.create_function(move |lua, ()| -> LuaResult<LuaValue> {
        let inventory_data = lua.create_table()?;
        let inventory = bot_clone.inventory.read().unwrap();

        inventory_data.set("size", inventory.size)?;
        inventory_data.set("item_count", inventory.item_count)?;

        let items_table = lua.create_table()?;
        for (_, (key, item)) in inventory.items.clone().into_iter().enumerate() {
            let item_table = lua.create_table()?;
            item_table.set("id", item.id)?;
            item_table.set("amount", item.amount)?;
            items_table.set(key, item_table)?;
        }
        inventory_data.set("items", items_table)?;

        Ok(LuaValue::Table(inventory_data))
    }).unwrap();
    lua.globals().set("get_inventory", get_inventory).unwrap();

    let bot_clone = bot.clone();
    let get_tile = lua.create_function(move |lua, (x, y): (u32, u32)| -> LuaResult<LuaValue> {
        let tile_data = lua.create_table()?;
        let world = bot_clone.world.read().unwrap();
        let tile = world.get_tile(x, y).unwrap();
        let tile_clone = tile.clone();
        let bot_clone = bot_clone.clone();

        tile_data.set("fg", tile.foreground_item_id)?;
        tile_data.set("bg", tile.background_item_id)?;
        tile_data.set("pbi", tile.parent_block_index)?;
        tile_data.set("flags", tile.flags)?;
        tile_data.set("x", tile.x)?;
        tile_data.set("y", tile.y)?;

        let tile_harvestable_func = lua.create_function(move |_, ()| -> LuaResult<bool> {
            let world = bot_clone.world.read().unwrap();
            let harvestable = world.is_tile_harvestable(&tile_clone);
            Ok(harvestable)
        })?;

        tile_data.set("harvestable", tile_harvestable_func)?;
        Ok(LuaValue::Table(tile_data))
    }).unwrap();
    lua.globals().set("get_tile", get_tile).unwrap();
}