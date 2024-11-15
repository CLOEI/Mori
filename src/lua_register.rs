use crate::core::Bot;
use mlua::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn register(lua: &Lua, bot: &Arc<Bot>) -> LuaResult<()> {
    let bot_table = lua.create_table()?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "walk",
        |bot, (x, y, ap): (i32, i32, bool)| {
            bot.walk(x, y, ap);
            Ok(())
        },
    )?;

    register_bot_function(lua, bot.clone(), &bot_table, "leave", |bot, (): ()| {
        bot.leave();
        Ok(())
    })?;

    register_bot_function(lua, bot.clone(), &bot_table, "disconnect", |bot, (): ()| {
        bot.disconnect();
        Ok(())
    })?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "is_in_world",
        |bot, (): ()| {
            let in_world = bot.is_inworld();
            Ok(in_world)
        },
    )?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "place",
        |bot, (offset_x, offset_y, item_id): (i32, i32, u32)| {
            bot.place(offset_x, offset_y, item_id);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "punch",
        |bot, (offset_x, offset_y): (i32, i32)| {
            bot.punch(offset_x, offset_y);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "wrench",
        |bot, (offset_x, offset_y): (i32, i32)| {
            bot.wrench(offset_x, offset_y);
            Ok(())
        },
    )?;

    register_bot_function(lua, bot.clone(), &bot_table, "wear", |bot, item_id: u32| {
        bot.wear(item_id);
        Ok(())
    })?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "warp",
        |bot, world_name: String| {
            bot.warp(world_name);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "talk",
        |bot, message: String| {
            bot.talk(message);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "find_path",
        |bot, (x, y): (u32, u32)| {
            bot.find_path(x, y);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "drop",
        |bot, (item_id, amount): (u32, u32)| {
            bot.drop_item(item_id, amount);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        bot.clone(),
        &bot_table,
        "trash",
        |bot, (item_id, amount): (u32, u32)| {
            bot.trash_item(item_id, amount);
            Ok(())
        },
    )?;

    bot_table.set(
        "sleep",
        lua.create_function(|_, ms: u64| {
            thread::sleep(Duration::from_millis(ms));
            Ok(())
        })?,
    )?;

    register_world_api(lua, bot.clone(), &bot_table)?;
    register_local_api(lua, bot.clone(), &bot_table)?;
    register_inventory_api(lua, bot.clone(), &bot_table)?;
    register_tile_api(lua, bot.clone(), &bot_table)?;

    lua.globals().set("bot", bot_table)?;

    Ok(())
}

fn register_bot_function<'lua, F, A, R>(
    lua: &'lua Lua,
    bot: Arc<Bot>,
    bot_table: &LuaTable<'lua>,
    name: &str,
    func: F,
) -> LuaResult<()>
where
    F: 'static + Send + Fn(&Arc<Bot>, A) -> LuaResult<R>,
    A: FromLuaMulti<'lua>,
    R: IntoLuaMulti<'lua>,
{
    let bot_clone = bot.clone();
    let lua_func = lua.create_function(move |_, args: A| func(&bot_clone, args))?;
    bot_table.set(name, lua_func)
}

fn register_world_api<'lua>(
    lua: &'lua Lua,
    bot: Arc<Bot>,
    bot_table: &LuaTable<'lua>,
) -> LuaResult<()> {
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
            let bot_clone_inner = bot_clone.clone();
            let tile_table = lua.create_table()?;

            let tile_harvestable_func = lua.create_function(move |_, ()| -> LuaResult<bool> {
                let world = bot_clone_inner.world.read().unwrap();
                let harvestable = world.is_tile_harvestable(&tile_clone);
                Ok(harvestable)
            })?;

            tile_table.set("fg", tile.foreground_item_id)?;
            tile_table.set("bg", tile.background_item_id)?;
            tile_table.set("pbi", tile.parent_block_index)?;
            // tile_table.set("flags", tile.flags)?;
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
            let dropped_table_entry = lua.create_table()?;
            dropped_table_entry.set("id", dropped.id)?;
            dropped_table_entry.set("x", dropped.x)?;
            dropped_table_entry.set("y", dropped.y)?;
            dropped_table_entry.set("count", dropped.count)?;
            dropped_table_entry.set("flags", dropped.flags)?;
            dropped_table_entry.set("uid", dropped.uid)?;
            dropped_items_table.set(i + 1, dropped_table_entry)?;
        }
        dropped_table.set("items", dropped_items_table)?;
        world_data.set("dropped", dropped_table)?;

        world_data.set("base_weather", world.base_weather)?;
        world_data.set("current_weather", world.current_weather)?;
        world_data.set("is_error", world.is_error)?;

        Ok(LuaValue::Table(world_data))
    })?;
    bot_table.set("get_world", get_world)?;
    Ok(())
}

fn register_local_api<'lua>(
    lua: &'lua Lua,
    bot: Arc<Bot>,
    bot_table: &LuaTable<'lua>,
) -> LuaResult<()> {
    let bot_clone = bot.clone();
    let get_local = lua.create_function(move |lua, ()| -> LuaResult<LuaValue> {
        let local_data = lua.create_table()?;
        let position = bot_clone.position.lock().unwrap();
        let state = bot_clone.state.lock().unwrap();

        local_data.set("x", position.x)?;
        local_data.set("y", position.y)?;
        local_data.set("net_id", state.net_id)?;
        local_data.set("level", state.level)?;
        local_data.set("gems", state.gems)?;

        Ok(LuaValue::Table(local_data))
    })?;
    bot_table.set("get_local", get_local)?;
    Ok(())
}

fn register_inventory_api<'lua>(
    lua: &'lua Lua,
    bot: Arc<Bot>,
    bot_table: &LuaTable<'lua>,
) -> LuaResult<()> {
    let bot_clone = bot.clone();
    let get_inventory = lua.create_function(move |lua, ()| -> LuaResult<LuaValue> {
        let inventory_data = lua.create_table()?;
        let inventory = bot_clone.inventory.lock().unwrap();

        inventory_data.set("size", inventory.size)?;
        inventory_data.set("item_count", inventory.item_count)?;

        let items_table = lua.create_table()?;
        for (key, item) in inventory.items.clone() {
            let item_table = lua.create_table()?;
            item_table.set("id", item.id)?;
            item_table.set("amount", item.amount)?;
            items_table.set(key, item_table)?;
        }
        inventory_data.set("items", items_table)?;

        Ok(LuaValue::Table(inventory_data))
    })?;
    bot_table.set("get_inventory", get_inventory)?;
    Ok(())
}

fn register_tile_api<'lua>(
    lua: &'lua Lua,
    bot: Arc<Bot>,
    bot_table: &LuaTable<'lua>,
) -> LuaResult<()> {
    let bot_clone = bot.clone();
    let get_tile = lua.create_function(move |lua, (x, y): (u32, u32)| -> LuaResult<LuaValue> {
        let tile_data = lua.create_table()?;
        let world = bot_clone.world.read().unwrap();
        let tile = match world.get_tile(x, y) {
            Some(t) => t.clone(),
            None => {
                return Err(LuaError::RuntimeError(format!(
                    "Tile at ({}, {}) not found",
                    x, y
                )))
            }
        };
        let tile_clone = tile.clone();
        let bot_clone_inner = bot_clone.clone();

        tile_data.set("fg", tile.foreground_item_id)?;
        tile_data.set("bg", tile.background_item_id)?;
        tile_data.set("pbi", tile.parent_block_index)?;
        // tile_data.set("flags", tile.flags)?;
        tile_data.set("x", tile.x)?;
        tile_data.set("y", tile.y)?;

        let tile_harvestable_func = lua.create_function(move |_, ()| -> LuaResult<bool> {
            let world = bot_clone_inner.world.read().unwrap();
            let harvestable = world.is_tile_harvestable(&tile_clone);
            Ok(harvestable)
        })?;

        tile_data.set("harvestable", tile_harvestable_func)?;
        Ok(LuaValue::Table(tile_data))
    })?;
    bot_table.set("get_tile", get_tile)?;
    Ok(())
}
