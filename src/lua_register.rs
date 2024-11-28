use crate::core::Bot;
use crate::types::epacket_type::EPacketType;
use mlua::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn register_bot_function<'lua, F, A, R>(
    lua: &'lua Lua,
    bot: &Arc<Bot>,
    bot_table: &LuaTable,
    name: &str,
    func: F,
) -> LuaResult<()>
where
    F: 'static + Send + Fn(&Arc<Bot>, A) -> LuaResult<R>,
    A: FromLuaMulti,
    R: IntoLuaMulti,
{
    let bot_clone = bot.clone();
    let lua_func = lua.create_function(move |_, args: A| func(&bot_clone, args))?;
    bot_table.set(name, lua_func)
}

pub fn register(lua: &Lua, bot: &Arc<Bot>) -> LuaResult<()> {
    let globals = lua.globals();

    register_bot_function(lua, &bot, &globals, "sleep", |_, ms: u64| {
        thread::sleep(Duration::from_millis(ms));
        Ok(())
    })?;

    register_bot_function(lua, &bot, &globals, "enter_door", |bot, ()| {
        bot.enter_door();
        Ok(())
    })?;

    register_bot_function(lua, &bot, &globals, "accept_access", |bot, ()| {
        bot.accept_access();
        Ok(())
    })?;

    register_bot_function(lua, &bot, &globals, "leave", |bot, ()| {
        bot.leave();
        Ok(())
    })?;

    register_bot_function(lua, &bot, &globals, "collect", |bot, ()| {
        bot.collect();
        Ok(())
    })?;

    register_bot_function(lua, &bot, &globals, "disconnect", |bot, ()| {
        bot.disconnect();
        Ok(())
    })?;

    register_bot_function(
        lua,
        &bot,
        &globals,
        "wrench_player",
        |bot, player_id: u32| {
            bot.wrench_player(player_id);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        &bot,
        &globals,
        "wrench",
        |bot, (offset_x, offset_y): (i32, i32)| {
            bot.wrench(offset_x, offset_y);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        &bot,
        &globals,
        "punch",
        |bot, (offset_x, offset_y): (i32, i32)| {
            bot.punch(offset_x, offset_y);
            Ok(())
        },
    )?;

    register_bot_function(lua, &bot, &globals, "wear", |bot, item_id: u32| {
        bot.wear(item_id);
        Ok(())
    })?;

    register_bot_function(lua, &bot, &globals, "talk", |bot, message: String| {
        bot.talk(message);
        Ok(())
    })?;

    register_bot_function(lua, &bot, &globals, "warp", |bot, world_name: String| {
        bot.warp(world_name);
        Ok(())
    })?;

    // Register functions with multiple arguments
    register_bot_function(
        lua,
        &bot,
        &globals,
        "trash",
        |bot, (item_id, amount): (u32, u32)| {
            bot.trash_item(item_id, amount);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        &bot,
        &globals,
        "drop",
        |bot, (item_id, amount): (u32, u32)| {
            bot.drop_item(item_id, amount);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        &bot,
        &globals,
        "find_path",
        |bot, (x, y): (u32, u32)| {
            bot.find_path(x, y);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        &bot,
        &globals,
        "walk",
        |bot, (offset_x, offset_y): (i32, i32)| {
            bot.walk(offset_x, offset_y, false);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        &bot,
        &globals,
        "place",
        |bot, (offset_x, offset_y, item_id): (i32, i32, u32)| {
            bot.place(offset_x, offset_y, item_id);
            Ok(())
        },
    )?;

    register_bot_function(
        lua,
        &bot,
        &globals,
        "send_packet",
        |bot, (pkt_type, data): (u32, String)| {
            let packet = EPacketType::from(pkt_type);
            bot.send_packet(packet, data);
            Ok(())
        },
    )?;

    register_bot_function(lua, &bot, &globals, "is_inworld", |bot, ()| {
        let is_inworld = bot.is_inworld();
        Ok(is_inworld)
    })?;

    Ok(())
}
