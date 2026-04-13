use mlua::prelude::*;
use mlua::MetaMethod;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::bot::BotEventRaw;
use crate::protocol::packet::{GamePacketType, GameUpdatePacket, PacketFlags};
use crate::player::Player;
use crate::script_channel::{ScriptRequest as Req, ScriptReply as Rep};
use crate::protocol::variant::Variant;
use crate::world::{TileFlags, TileType, World};

use super::http::register_http_client;
use super::webhook::register_webhook;
use super::types::{
    BotProxy, LuaGameUpdatePacket, LuaInventory, LuaInventoryItem, LuaItemInfo,
    LuaLogin, LuaNetObject, LuaNpc, LuaPlayer, LuaTile, LuaVariant, LuaVariantList, LuaWorld,
};

fn sequence_index_value<F>(
    lua: &Lua,
    key: LuaValue,
    len: usize,
    get_value: &F,
) -> LuaResult<LuaValue>
where
    F: Fn(&Lua, usize) -> LuaResult<LuaValue>,
{
    let index = match key {
        LuaValue::Integer(i) if i >= 1 => (i - 1) as usize,
        LuaValue::Number(n) if n.fract() == 0.0 && n >= 1.0 => (n as usize).saturating_sub(1),
        _ => return Ok(LuaValue::Nil),
    };
    if index >= len {
        return Ok(LuaValue::Nil);
    }
    get_value(lua, index)
}

fn create_lazy_sequence<F>(lua: &Lua, len: usize, get_value: F) -> LuaResult<LuaTable>
where
    F: Fn(&Lua, usize) -> LuaResult<LuaValue> + Clone + 'static,
{
    let table = lua.create_table()?;
    table.raw_set("__mori_sequence", true)?;

    let metatable = lua.create_table()?;

    metatable.set(
        MetaMethod::Len.name(),
        lua.create_function(move |_, _: LuaTable| Ok(len))?,
    )?;

    let get_value_index = get_value.clone();
    metatable.set(
        MetaMethod::Index.name(),
        lua.create_function(move |lua, (_table, key): (LuaTable, LuaValue)| {
            sequence_index_value(lua, key, len, &get_value_index)
        })?,
    )?;

    let get_value_pairs = get_value;
    metatable.set(
        MetaMethod::Pairs.name(),
        lua.create_function(move |lua, table: LuaTable| {
            let get_value = get_value_pairs.clone();
            let iter = lua.create_function(move |lua, (_seq, last): (LuaTable, LuaValue)| {
                let next_index = match last {
                    LuaValue::Integer(i) if i >= 0 => i + 1,
                    LuaValue::Nil => 1,
                    _ => 1,
                };
                if next_index as usize > len {
                    return Ok((LuaValue::Nil, LuaValue::Nil));
                }
                let item = sequence_index_value(lua, LuaValue::Integer(next_index), len, &get_value)?;
                if matches!(item, LuaValue::Nil) {
                    Ok((LuaValue::Nil, LuaValue::Nil))
                } else {
                    Ok((LuaValue::Integer(next_index), item))
                }
            })?;
            Ok((iter, table, LuaValue::Nil))
        })?,
    )?;

    let _ = table.set_metatable(Some(metatable));
    Ok(table)
}

fn tile_value(lua: &Lua, tile: TileTypeSource) -> LuaResult<LuaValue> {
    Ok(LuaValue::UserData(lua.create_userdata(LuaTile(tile.into_tile()))?))
}

enum TileTypeSource {
    Tile(crate::world::Tile),
}

impl TileTypeSource {
    fn into_tile(self) -> crate::world::Tile {
        match self {
            TileTypeSource::Tile(tile) => tile,
        }
    }
}

fn create_lazy_tile_sequence(lua: &Lua, world: Arc<World>) -> LuaResult<LuaTable> {
    let len = world.tile_map.tiles.len();
    create_lazy_sequence(lua, len, move |lua, index| {
        tile_value(lua, TileTypeSource::Tile(world.tile_map.tiles[index].clone()))
    })
}

fn create_lazy_object_sequence(lua: &Lua, world: Arc<World>) -> LuaResult<LuaTable> {
    let len = world.objects.len();
    create_lazy_sequence(lua, len, move |lua, index| {
        Ok(LuaValue::UserData(lua.create_userdata(LuaNetObject(world.objects[index].clone()))?))
    })
}

fn create_lazy_player_sequence(lua: &Lua, players: Arc<Vec<Player>>) -> LuaResult<LuaTable> {
    let len = players.len();
    create_lazy_sequence(lua, len, move |lua, index| {
        Ok(LuaValue::UserData(lua.create_userdata(LuaPlayer(players[index].clone()))?))
    })
}

fn create_lazy_npc_sequence(lua: &Lua, world: Arc<World>) -> LuaResult<LuaTable> {
    let len = world.npcs.len();
    create_lazy_sequence(lua, len, move |lua, index| {
        Ok(LuaValue::UserData(lua.create_userdata(LuaNpc(world.npcs[index].clone()))?))
    })
}

impl LuaUserData for BotProxy {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, p| {
            Ok(p.state.read().unwrap().username.clone())
        });
        fields.add_field_method_get("status", |_, p| {
            Ok(p.state.read().unwrap().status.to_string())
        });
        fields.add_field_method_get("gem_count", |_, p| {
            Ok(p.state.read().unwrap().gems)
        });
        fields.add_field_method_get("auto_collect", |_, p| {
            match p.request(Req::GetAutoCollect) { Rep::Bool(v) => Ok(v), _ => Ok(false) }
        });
        fields.add_field_method_set("auto_collect", |_, p, v: bool| {
            p.request(Req::SetAutoCollect { enabled: v });
            Ok(())
        });
        fields.add_field_method_get("ignore_gems", |_, p| {
            match p.request(Req::GetIgnoreGems) { Rep::Bool(v) => Ok(v), _ => Ok(false) }
        });
        fields.add_field_method_set("ignore_gems", |_, p, v: bool| {
            p.request(Req::SetIgnoreGems { enabled: v });
            Ok(())
        });
        fields.add_field_method_get("ignore_essences", |_, p| {
            match p.request(Req::GetIgnoreEssences) { Rep::Bool(v) => Ok(v), _ => Ok(false) }
        });
        fields.add_field_method_set("ignore_essences", |_, p, v: bool| {
            p.request(Req::SetIgnoreEssences { enabled: v });
            Ok(())
        });
        fields.add_field_method_get("auto_leave_on_mod", |_, p| {
            match p.request(Req::GetAutoLeaveOnMod) { Rep::Bool(v) => Ok(v), _ => Ok(false) }
        });
        fields.add_field_method_set("auto_leave_on_mod", |_, p, v: bool| {
            p.request(Req::SetAutoLeaveOnMod { enabled: v });
            Ok(())
        });
        fields.add_field_method_get("auto_ban", |_, p| {
            match p.request(Req::GetAutoBan) { Rep::Bool(v) => Ok(v), _ => Ok(false) }
        });
        fields.add_field_method_set("auto_ban", |_, p, v: bool| {
            p.request(Req::SetAutoBan { enabled: v });
            Ok(())
        });
        fields.add_field_method_get("collect_interval", |_, p| {
            match p.request(Req::GetCollectInterval) { Rep::U32(v) => Ok(v), _ => Ok(500) }
        });
        fields.add_field_method_set("collect_interval", |_, p, v: u64| {
            p.request(Req::SetCollectInterval { ms: v });
            Ok(())
        });
        fields.add_field_method_get("collect_range", |_, p| {
            match p.request(Req::GetCollectRange) { Rep::U32(v) => Ok(v), _ => Ok(3) }
        });
        fields.add_field_method_set("collect_range", |_, p, v: u8| {
            p.request(Req::SetCollectRange { tiles: v });
            Ok(())
        });
        fields.add_field_method_get("collect_path_check", |_, p| {
            match p.request(Req::GetCollectPathCheck) { Rep::Bool(v) => Ok(v), _ => Ok(true) }
        });
        fields.add_field_method_set("collect_path_check", |_, p, v: bool| {
            p.request(Req::SetCollectPathCheck { enabled: v });
            Ok(())
        });
        fields.add_field_method_get("reconnect_interval", |_, p| {
            match p.request(Req::GetReconnectInterval) { Rep::U32(v) => Ok(v), _ => Ok(0) }
        });
        fields.add_field_method_set("reconnect_interval", |_, p, v: u64| {
            p.request(Req::SetReconnectInterval { ms: v });
            Ok(())
        });
        fields.add_field_method_get("place_delay", |_, p| {
            match p.request(Req::GetPlaceDelay) { Rep::U32(v) => Ok(v), _ => Ok(500) }
        });
        fields.add_field_method_set("place_delay", |_, p, v: u64| {
            p.request(Req::SetPlaceDelay { ms: v });
            Ok(())
        });
        fields.add_field_method_get("walk_delay", |_, p| {
            match p.request(Req::GetWalkDelay) { Rep::U32(v) => Ok(v), _ => Ok(500) }
        });
        fields.add_field_method_set("walk_delay", |_, p, v: u64| {
            p.request(Req::SetWalkDelay { ms: v });
            Ok(())
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // ── Getters ────────────────────────────────────────────────────────────
        methods.add_method("getWorld", |_, p, ()| {
            match p.request(Req::GetWorld) {
                Rep::World(Some(snap)) => Ok(Some(LuaWorld {
                    world:         snap.world,
                    players:       snap.players,
                    local_net_id:  snap.local_net_id,
                    local_user_id: snap.local_user_id,
                    local_name:    snap.local_name,
                    local_pos:     snap.local_pos,
                })),
                _ => Ok(None),
            }
        });

        methods.add_method("getInventory", |_, p, ()| {
            match p.request(Req::GetInventory) {
                Rep::Inventory(inv) => Ok(LuaInventory(inv)),
                _                   => Ok(LuaInventory(crate::inventory::Inventory::default())),
            }
        });

        methods.add_method("getLogin", |_, p, ()| {
            Ok(LuaLogin { mac: p.state.read().unwrap().mac.clone() })
        });

        methods.add_method("getPing", |_, p, ()| {
            Ok(p.state.read().unwrap().ping_ms)
        });

        methods.add_method("getLocal", |_, p, ()| {
            match p.request(Req::GetLocal) {
                Rep::Local(snap) => Ok(LuaPlayer(Player {
                    net_id:     snap.net_id,
                    user_id:    snap.user_id,
                    name:       snap.username,
                    country:    String::new(),
                    position:   (snap.pos_x, snap.pos_y),
                    avatar:     String::new(),
                    online_id:  String::new(),
                    e_id:       String::new(),
                    ip:         String::new(),
                    col_rect:   String::new(),
                    title_icon: String::new(),
                    m_state:    0,
                    sm_state:   0,
                    invisible:  false,
                })),
                _ => Err(LuaError::runtime("getLocal failed")),
            }
        });

        methods.add_method("getCaptcha", |_, _p, ()| Ok(None::<String>));

        // ── State queries ──────────────────────────────────────────────────────
        methods.add_method("isInWorld", |_, p, name: Option<String>| {
            match p.request(Req::IsInWorld { name }) { Rep::Bool(v) => Ok(v), _ => Ok(false) }
        });

        methods.add_method("isInTile", |_, p, (x, y): (u32, u32)| {
            match p.request(Req::IsInTile { x, y }) { Rep::Bool(v) => Ok(v), _ => Ok(false) }
        });

        // ── Network ────────────────────────────────────────────────────────────
        methods.add_method("connect", |_, p, ()| {
            p.request(Req::Reconnect);
            Ok(true)
        });

        methods.add_method("disconnect", |_, p, ()| {
            p.request(Req::Disconnect);
            Ok(())
        });

        methods.add_method("sendRaw", |_, p, pkt: LuaAnyUserData| {
            let pkt = pkt.borrow::<LuaGameUpdatePacket>()?.0.clone();
            p.request(Req::SendRaw { pkt });
            Ok(())
        });

        methods.add_method("sendPacket", |_, p, (ptype, text): (u8, String)| {
            p.request(Req::SendPacket { ptype, text });
            Ok(())
        });

        // ── World actions ──────────────────────────────────────────────────────
        methods.add_method("warp", |_, p, (name, id): (String, Option<String>)| {
            p.request(Req::Warp { name, id: id.unwrap_or_default() });
            Ok(())
        });

        methods.add_method("say", |_, p, text: String| {
            p.request(Req::Say { text });
            Ok(())
        });

        methods.add_method("leaveWorld", |_, p, ()| {
            p.request(Req::LeaveWorld);
            Ok(())
        });

        methods.add_method("respawn", |_, p, ()| {
            p.request(Req::Respawn);
            Ok(())
        });

        methods.add_method("active", |_, p, (x, y): (i32, i32)| {
            p.request(Req::Active { tile_x: x, tile_y: y });
            Ok(())
        });

        methods.add_method("place", |_, p, (x, y, item): (i32, i32, u32)| {
            p.request(Req::Place { x, y, item });
            Ok(())
        });

        methods.add_method("hit", |_, p, (x, y): (i32, i32)| {
            p.request(Req::Hit { x, y });
            Ok(())
        });

        methods.add_method("wrench", |_, p, (x, y): (i32, i32)| {
            p.request(Req::Wrench { x, y });
            Ok(())
        });

        methods.add_method("wrenchPlayer", |_, p, net_id: u32| {
            p.request(Req::WrenchPlayer { net_id });
            Ok(())
        });

        methods.add_method("enter", |_, p, pass: Option<String>| {
            p.request(Req::Enter { pass });
            Ok(())
        });

        // ── Inventory ──────────────────────────────────────────────────────────
        methods.add_method("wear",      |_, p, item_id: u32|          { p.request(Req::Wear      { item_id });        Ok(()) });
        methods.add_method("unwear",    |_, p, item_id: u32|          { p.request(Req::Unwear    { item_id });        Ok(()) });
        methods.add_method("use",       |_, p, item_id: u32|          { p.request(Req::Wear      { item_id });        Ok(()) });
        methods.add_method("drop",      |_, p, (item_id, count): (u32, u32)| { p.request(Req::Drop  { item_id, count }); Ok(()) });
        methods.add_method("trash",     |_, p, (item_id, count): (u32, u32)| { p.request(Req::Trash { item_id, count }); Ok(()) });
        methods.add_method("fastDrop",  |_, p, (item_id, count): (u32, u32)| { p.request(Req::FastDrop  { item_id, count }); Ok(()) });
        methods.add_method("fastTrash", |_, p, (item_id, count): (u32, u32)| { p.request(Req::FastTrash { item_id, count }); Ok(()) });

        // ── Movement ───────────────────────────────────────────────────────────
        methods.add_method("moveTo", |_, p, (dx, dy): (i32, i32)| {
            let s = p.state.read().unwrap();
            let cx = s.pos_x as i32;
            let cy = s.pos_y as i32;
            drop(s);
            p.request(Req::Walk { tile_x: cx + dx, tile_y: cy + dy });
            Ok(())
        });

        methods.add_method("moveTile", |_, p, (x, y): (i32, i32)| {
            p.request(Req::Walk { tile_x: x, tile_y: y });
            Ok(())
        });

        methods.add_method("moveLeft", |_, p, range: Option<i32>| {
            let r = range.unwrap_or(1);
            let s = p.state.read().unwrap();
            let cx = s.pos_x as i32; let cy = s.pos_y as i32;
            drop(s);
            p.request(Req::Walk { tile_x: cx - r, tile_y: cy });
            Ok(())
        });

        methods.add_method("moveRight", |_, p, range: Option<i32>| {
            let r = range.unwrap_or(1);
            let s = p.state.read().unwrap();
            let cx = s.pos_x as i32; let cy = s.pos_y as i32;
            drop(s);
            p.request(Req::Walk { tile_x: cx + r, tile_y: cy });
            Ok(())
        });

        methods.add_method("moveUp", |_, p, range: Option<i32>| {
            let r = range.unwrap_or(1);
            let s = p.state.read().unwrap();
            let cx = s.pos_x as i32; let cy = s.pos_y as i32;
            drop(s);
            p.request(Req::Walk { tile_x: cx, tile_y: cy - r });
            Ok(())
        });

        methods.add_method("moveDown", |_, p, range: Option<i32>| {
            let r = range.unwrap_or(1);
            let s = p.state.read().unwrap();
            let cx = s.pos_x as i32; let cy = s.pos_y as i32;
            drop(s);
            p.request(Req::Walk { tile_x: cx, tile_y: cy + r });
            Ok(())
        });

        methods.add_method("setDirection", |_, p, facing_left: bool| {
            p.request(Req::SetDirection { facing_left });
            Ok(())
        });

        methods.add_method("setMac", |_, p, mac: String| {
            p.request(Req::SetMac { mac });
            Ok(())
        });

        methods.add_method("setAutoCollect", |_, p, enabled: bool| {
            p.request(Req::SetAutoCollect { enabled });
            Ok(())
        });

        methods.add_method("setIgnoreGems", |_, p, enabled: bool| {
            p.request(Req::SetIgnoreGems { enabled });
            Ok(())
        });

        methods.add_method("setIgnoreEssences", |_, p, enabled: bool| {
            p.request(Req::SetIgnoreEssences { enabled });
            Ok(())
        });

        methods.add_method("setAutoLeaveOnMod", |_, p, enabled: bool| {
            p.request(Req::SetAutoLeaveOnMod { enabled });
            Ok(())
        });

        methods.add_method("setAutoBan", |_, p, enabled: bool| {
            p.request(Req::SetAutoBan { enabled });
            Ok(())
        });

        // ── Pathfinding ────────────────────────────────────────────────────────
        methods.add_method("getPath", |lua, p, (x, y): (u32, u32)| {
            let nodes = match p.request(Req::GetPath { x, y }) {
                Rep::Path(v) => v,
                _            => vec![],
            };
            let table = lua.create_table()?;
            for (i, (nx, ny)) in nodes.iter().enumerate() {
                let t = lua.create_table()?;
                t.set("x", *nx)?;
                t.set("y", *ny)?;
                table.set(i + 1, t)?;
            }
            Ok(table)
        });

        methods.add_method("findPath", |_, p, (x, y): (u32, u32)| {
            p.request(Req::FindPath { x, y });
            Ok(())
        });

        methods.add_method("collectObject", |_, p, (oid, range): (u32, f32)| {
            p.request(Req::CollectObject { uid: oid, range });
            Ok(())
        });

        methods.add_method("collect", |_, p, (range, interval): (f32, u64)| {
            match p.request(Req::Collect { range, interval_ms: interval }) {
                Rep::CollectCount(n) => Ok(n),
                _                   => Ok(0usize),
            }
        });

        // ── Script control ─────────────────────────────────────────────────────
        methods.add_method("stopScript", |_, _p, ()| Ok(()));
    }
}

// ── LuaWorld ──────────────────────────────────────────────────────────────────

impl LuaUserData for LuaWorld {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name",       |_, w| Ok(w.world.tile_map.world_name.clone()));
        fields.add_field_method_get("x",          |_, w| Ok(w.world.tile_map.width));
        fields.add_field_method_get("y",          |_, w| Ok(w.world.tile_map.height));
        fields.add_field_method_get("tile_count", |_, w| Ok(w.world.tile_map.tiles.len()));
        fields.add_field_method_get("version",    |_, w| Ok(w.world.version));
        // Convenience table properties
        fields.add_field_method_get("tiles", |lua, w| {
            create_lazy_tile_sequence(lua, Arc::clone(&w.world))
        });
        fields.add_field_method_get("objects", |lua, w| {
            create_lazy_object_sequence(lua, Arc::clone(&w.world))
        });
        fields.add_field_method_get("players", |lua, w| {
            create_lazy_player_sequence(lua, Arc::clone(&w.players))
        });
        fields.add_field_method_get("npcs", |lua, w| {
            create_lazy_npc_sequence(lua, Arc::clone(&w.world))
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getTile", |_, w, (x, y): (u32, u32)| {
            Ok(w.world.get_tile(x, y).cloned().map(LuaTile))
        });

        methods.add_method("getTiles", |lua, w, ()| {
            create_lazy_tile_sequence(lua, Arc::clone(&w.world))
        });

        methods.add_method("getObject", |_, w, oid: u32| {
            Ok(w.world.objects.iter().find(|o| o.uid == oid).cloned().map(LuaNetObject))
        });

        methods.add_method("getObjects", |lua, w, ()| {
            create_lazy_object_sequence(lua, Arc::clone(&w.world))
        });

        methods.add_method("getPlayer", |_, w, key: LuaValue| {
            let found = match &key {
                LuaValue::Integer(n) => w.players.iter().find(|p| p.net_id == *n as u32),
                LuaValue::Number(n)  => w.players.iter().find(|p| p.net_id == *n as u32),
                LuaValue::String(s)  => {
                    let name = s.to_str().map(|b| b.to_lowercase()).unwrap_or_default();
                    w.players.iter().find(|p| p.name.to_lowercase() == name)
                }
                _ => None,
            };
            Ok(found.cloned().map(LuaPlayer))
        });

        methods.add_method("getPlayers", |lua, w, ()| {
            create_lazy_player_sequence(lua, Arc::clone(&w.players))
        });

        methods.add_method("getLocal", |_, w, ()| {
            let fake = Player {
                net_id:     w.local_net_id,
                user_id:    w.local_user_id,
                name:       w.local_name.clone(),
                country:    String::new(),
                position:   w.local_pos,
                avatar:     String::new(),
                online_id:  String::new(),
                e_id:       String::new(),
                ip:         String::new(),
                col_rect:   String::new(),
                title_icon: String::new(),
                m_state:    0,
                sm_state:   0,
                invisible:  false,
            };
            Ok(LuaPlayer(fake))
        });

        methods.add_method("isValidPosition", |_, w, (x, y): (i32, i32)| {
            if x < 0 || y < 0 { return Ok(false); }
            Ok((x as u32) < w.world.tile_map.width && (y as u32) < w.world.tile_map.height)
        });

        methods.add_method("getTileParent", |_, w, tile: LuaAnyUserData| {
            let tile = tile.borrow::<LuaTile>()?;
            if !tile.0.flags.contains(TileFlags::HAS_PARENT) {
                return Ok(None);
            }
            let idx = tile.0.parent_block as usize;
            Ok(w.world.tile_map.tiles.get(idx).cloned().map(LuaTile))
        });

        methods.add_method("getNPC", |_, w, id: u8| {
            Ok(w.world.npcs.iter().find(|n| n.id == id).cloned().map(LuaNpc))
        });

        methods.add_method("getNPCs", |lua, w, ()| {
            create_lazy_npc_sequence(lua, Arc::clone(&w.world))
        });
    }
}

// ── LuaInventory ──────────────────────────────────────────────────────────────

impl LuaUserData for LuaInventory {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("itemcount", |_, inv| Ok(inv.0.item_count));
        fields.add_field_method_get("slotcount", |_, inv| Ok(inv.0.size));
        fields.add_field_method_get("items", |lua, inv| {
            let t = lua.create_table()?;
            for (i, item) in inv.0.items.values().enumerate() {
                t.set(i + 1, LuaInventoryItem(item.clone()))?;
            }
            Ok(t)
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getItem", |_, inv, id: LuaValue| {
            let key = match &id {
                LuaValue::Integer(n) => *n as u16,
                LuaValue::Number(n)  => *n as u16,
                LuaValue::String(s)  => s.to_str().map(|b| b.parse().unwrap_or(0)).unwrap_or(0),
                _ => return Ok(None),
            };
            Ok(inv.0.items.get(&key).cloned().map(LuaInventoryItem))
        });

        methods.add_method("getItems", |lua, inv, ()| {
            let t = lua.create_table()?;
            for (i, item) in inv.0.items.values().enumerate() {
                t.set(i + 1, LuaInventoryItem(item.clone()))?;
            }
            Ok(t)
        });

        methods.add_method("findItem", |_, inv, id: u16| {
            Ok(inv.0.items.get(&id).map(|i| i.amount as u32).unwrap_or(0))
        });

        methods.add_method("getItemCount", |_, inv, id: u16| {
            Ok(inv.0.items.get(&id).map(|i| i.amount as u32).unwrap_or(0))
        });

        methods.add_method("canCollect", |_, inv, id: u16| {
            Ok(inv.0.can_collect(id))
        });
    }
}

// ── LuaInventoryItem ──────────────────────────────────────────────────────────

impl LuaUserData for LuaInventoryItem {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id",       |_, i| Ok(i.0.id));
        fields.add_field_method_get("count",    |_, i| Ok(i.0.amount));
        fields.add_field_method_get("isActive", |_, i| Ok(i.0.flag != 0));
    }
}

// ── LuaPlayer ─────────────────────────────────────────────────────────────────

impl LuaUserData for LuaPlayer {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name",       |_, p| Ok(p.0.name.clone()));
        fields.add_field_method_get("country",    |_, p| Ok(p.0.country.clone()));
        fields.add_field_method_get("netid",      |_, p| Ok(p.0.net_id));
        fields.add_field_method_get("userid",     |_, p| Ok(p.0.user_id));
        fields.add_field_method_get("posx",       |_, p| Ok(p.0.position.0));
        fields.add_field_method_get("posy",       |_, p| Ok(p.0.position.1));
        fields.add_field_method_get("avatarFlags",|_, p| Ok(p.0.m_state));
        fields.add_field_method_get("roleicon",   |_, p| Ok(p.0.title_icon.clone()));
    }
}

// ── LuaTile ───────────────────────────────────────────────────────────────────

impl LuaUserData for LuaTile {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("fg",         |_, t| Ok(t.0.fg_item_id));
        fields.add_field_method_get("foreground", |_, t| Ok(t.0.fg_item_id));
        fields.add_field_method_get("bg",         |_, t| Ok(t.0.bg_item_id));
        fields.add_field_method_get("background", |_, t| Ok(t.0.bg_item_id));
        fields.add_field_method_get("x",          |_, t| Ok(t.0.x));
        fields.add_field_method_get("y",          |_, t| Ok(t.0.y));
        fields.add_field_method_get("flags",      |_, t| Ok(t.0.flags_raw));
        fields.add_field_method_get("parent",     |_, t| Ok(t.0.parent_block));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("hasExtra", |_, t, ()| {
            Ok(t.0.flags.contains(TileFlags::HAS_EXTRA_DATA))
        });

        methods.add_method("getExtra", |lua, t, ()| {
            if !t.0.flags.contains(TileFlags::HAS_EXTRA_DATA) {
                return Ok(LuaValue::Nil);
            }
            let tbl = lua.create_table()?;
            match &t.0.tile_type {
                TileType::Sign { label } => {
                    tbl.set("type", "sign")?;
                    tbl.set("label", label.clone())?;
                }
                TileType::Door { label, flags } => {
                    tbl.set("type", "door")?;
                    tbl.set("label", label.clone())?;
                    tbl.set("flags", *flags)?;
                }
                TileType::Lock { settings, owner_uid, access_count, .. } => {
                    tbl.set("type", "lock")?;
                    tbl.set("settings", *settings)?;
                    tbl.set("owner_uid", *owner_uid)?;
                    tbl.set("access_count", *access_count)?;
                }
                TileType::Seed { age: time_passed, item_on_tree } => {
                    tbl.set("type", "seed")?;
                    tbl.set("time_passed", *time_passed)?;
                    tbl.set("item_on_tree", *item_on_tree)?;
                }
                TileType::Mannequin { 
                    label, hat, shirt, pants, boots,
                    face, hand, back, hair, neck, .. 
                } => {
                    tbl.set("type", "mannequin")?;
                    tbl.set("label", label.clone())?;
                    tbl.set("hat", *hat)?;
                    tbl.set("shirt", *shirt)?;
                    tbl.set("pants", *pants)?;
                    tbl.set("boots", *boots)?;
                    tbl.set("face", *face)?;
                    tbl.set("hand", *hand)?;
                    tbl.set("back", *back)?;
                    tbl.set("hair", *hair)?;
                    tbl.set("neck", *neck)?;
                }
                TileType::WeatherMachine { settings } => {
                    tbl.set("type", "weather_machine")?;
                    tbl.set("settings", *settings)?;
                }
                TileType::Dice { symbol } => {
                    tbl.set("type", "dice")?;
                    tbl.set("symbol", *symbol)?;
                }
                _ => {
                    tbl.set("type", "unknown")?;
                }
            }
            Ok(LuaValue::Table(tbl))
        });

        methods.add_method("canHarvest", |_, t, ()| {
            Ok(matches!(&t.0.tile_type, TileType::Seed { item_on_tree, .. } if *item_on_tree > 0))
        });

        methods.add_method("hasFlag", |_, t, flag: u16| {
            Ok(t.0.flags_raw & flag != 0)
        });
    }
}

// ── LuaNetObject ─────────────────────────────────────────────────────────────

impl LuaUserData for LuaNetObject {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id",    |_, o| Ok(o.0.item_id));
        fields.add_field_method_get("x",     |_, o| Ok(o.0.x));
        fields.add_field_method_get("y",     |_, o| Ok(o.0.y));
        fields.add_field_method_get("count", |_, o| Ok(o.0.count));
        fields.add_field_method_get("flags", |_, o| Ok(o.0.flags));
        fields.add_field_method_get("oid",   |_, o| Ok(o.0.uid));
    }
}

// ── LuaNpc ───────────────────────────────────────────────────────────────────

impl LuaUserData for LuaNpc {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("type",  |_, n| Ok(n.0.npc_type as u8));
        fields.add_field_method_get("id",    |_, n| Ok(n.0.id));
        fields.add_field_method_get("x",     |_, n| Ok(n.0.x));
        fields.add_field_method_get("y",     |_, n| Ok(n.0.y));
        fields.add_field_method_get("destx", |_, n| Ok(n.0.dest_x));
        fields.add_field_method_get("desty", |_, n| Ok(n.0.dest_y));
        fields.add_field_method_get("var",   |_, n| Ok(n.0.var));
        fields.add_field_method_get("unk",   |_, n| Ok(n.0.unk1));
    }
}

// ── LuaItemInfo ──────────────────────────────────────────────────────────────

impl LuaUserData for LuaItemInfo {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id",                 |_, i| Ok(i.0.id));
        fields.add_field_method_get("name",               |_, i| Ok(i.0.name.clone()));
        fields.add_field_method_get("action_type",        |_, i| Ok(i.0.action_type));
        fields.add_field_method_get("collision_type",     |_, i| Ok(i.0.collision_type));
        fields.add_field_method_get("clothing_type",      |_, i| Ok(i.0.clothing_type));
        fields.add_field_method_get("rarity",             |_, i| Ok(i.0.rarity));
        fields.add_field_method_get("grow_time",          |_, i| Ok(i.0.grow_time));
        fields.add_field_method_get("drop_chance",        |_, i| Ok(i.0.drop_chance));
        fields.add_field_method_get("texture",            |_, i| Ok(i.0.texture_file_name.clone()));
        fields.add_field_method_get("texture_hash",       |_, i| Ok(i.0.texture_hash));
        fields.add_field_method_get("texture_x",          |_, i| Ok(i.0.texture_x));
        fields.add_field_method_get("texture_y",          |_, i| Ok(i.0.texture_y));
        fields.add_field_method_get("seed_color",         |_, i| Ok(i.0.base_color));
        fields.add_field_method_get("seed_overlay_color", |_, i| Ok(i.0.overlay_color));
        fields.add_field_method_get("null_Item",          |_, i| {
            Ok(i.0.name.to_lowercase().contains("null"))
        });
        fields.add_field_method_get("strength",           |_, i| Ok(i.0.block_health / 6));
    }
}

// ── LuaGameUpdatePacket ───────────────────────────────────────────────────────

impl LuaUserData for LuaGameUpdatePacket {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        macro_rules! rw_u8 {
            ($name:literal, $field:ident) => {
                fields.add_field_method_get($name, |_, p| Ok(p.0.$field));
                fields.add_field_method_set($name, |_, p, v: u8| { p.0.$field = v; Ok(()) });
            };
        }
        macro_rules! rw_i32 {
            ($name:literal, $field:ident) => {
                fields.add_field_method_get($name, |_, p| Ok(p.0.$field));
                fields.add_field_method_set($name, |_, p, v: i32| { p.0.$field = v; Ok(()) });
            };
        }
        macro_rules! rw_u32 {
            ($name:literal, $field:ident) => {
                fields.add_field_method_get($name, |_, p| Ok(p.0.$field));
                fields.add_field_method_set($name, |_, p, v: u32| { p.0.$field = v; Ok(()) });
            };
        }
        macro_rules! rw_f32 {
            ($name:literal, $field:ident) => {
                fields.add_field_method_get($name, |_, p| Ok(p.0.$field));
                fields.add_field_method_set($name, |_, p, v: f32| { p.0.$field = v; Ok(()) });
            };
        }

        // type
        fields.add_field_method_get("type", |_, p| Ok(p.0.packet_type.as_u8()));
        fields.add_field_method_set("type", |_, p, v: u8| {
            p.0.packet_type = GamePacketType::from(v); Ok(())
        });

        rw_u8!("object_type", object_type);
        rw_u8!("count1",      jump_count);
        rw_u8!("count2",      animation_type);
        rw_u32!("netid",      net_id);
        rw_i32!("item",       target_net_id);

        // flags
        fields.add_field_method_get("flags", |_, p| Ok(p.0.flags.bits()));
        fields.add_field_method_set("flags", |_, p, v: u32| {
            p.0.flags = PacketFlags::from_bits_retain(v); Ok(())
        });

        rw_f32!("float_var",          float_variable);
        rw_u32!("int_data",           value);
        rw_f32!("vec_x",              vector_x);
        rw_f32!("pos_x",              vector_x);
        rw_f32!("vec_y",              vector_y);
        rw_f32!("pos_y",              vector_y);
        rw_f32!("vec2_x",             vector_x2);
        rw_f32!("pos2_x",             vector_x2);
        rw_f32!("vec2_y",             vector_y2);
        rw_f32!("pos2_y",             vector_y2);
        rw_f32!("particle_rotation",  particle_rotation);
        rw_i32!("int_x",              int_x);
        rw_i32!("int_y",              int_y);
    }
}

// ── LuaVariant ────────────────────────────────────────────────────────────────

impl LuaUserData for LuaVariant {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getType", |_, v, ()| Ok(match &v.0 {
            Variant::Float(_)      => 1u8,
            Variant::String(_)     => 2,
            Variant::Vec2(_, _)    => 3,
            Variant::Vec3(_, _, _) => 4,
            Variant::Unsigned(_)   => 5,
            Variant::Signed(_)     => 9,
            Variant::Unknown       => 0,
        }));

        methods.add_method("getString",   |_, v, ()| Ok(v.0.as_string()));
        methods.add_method("getInt",      |_, v, ()| Ok(v.0.as_int32()));
        methods.add_method("getFloat",    |_, v, ()| Ok(match &v.0 {
            Variant::Float(f) => *f, _ => 0.0f32,
        }));
        methods.add_method("getVector2", |lua, v, ()| {
            let (x, y) = v.0.as_vec2();
            let t = lua.create_table()?;
            t.set("x", x)?; t.set("y", y)?;
            Ok(t)
        });
        methods.add_method("getVector3", |lua, v, ()| {
            let (x, y, z) = match &v.0 {
                Variant::Vec3(x, y, z) => (*x, *y, *z), _ => (0.0, 0.0, 0.0),
            };
            let t = lua.create_table()?;
            t.set("x", x)?; t.set("y", y)?; t.set("z", z)?;
            Ok(t)
        });
        methods.add_method("print", |_, v, ()| Ok(v.0.as_string()));
    }
}

// ── LuaVariantList ────────────────────────────────────────────────────────────

impl LuaUserData for LuaVariantList {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |_, vl, idx: usize| {
            Ok(vl.0.get(idx).cloned().map(LuaVariant))
        });
        methods.add_method("print", |_, vl, ()| {
            let parts: Vec<String> = (0..)
                .map_while(|i| vl.0.get(i).map(|v| v.as_string()))
                .collect();
            Ok(parts.join(", "))
        });
    }
}

// ── LuaLogin ─────────────────────────────────────────────────────────────────

impl LuaUserData for LuaLogin {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("mac", |_, l| Ok(l.mac.clone()));
    }
}

// ── Public entry points ───────────────────────────────────────────────────────

/// Entry point called by the spawned script thread.
pub fn run_script_threaded(
    req_tx:    crossbeam_channel::Sender<crate::script_channel::ScriptRequest>,
    reply_rx:  crossbeam_channel::Receiver<crate::script_channel::ScriptReply>,
    event_rx:  crossbeam_channel::Receiver<crate::bot::BotEventRaw>,
    items:     std::sync::Arc<crate::items::ItemsDat>,
    state:     std::sync::Arc<std::sync::RwLock<crate::bot_state::BotState>>,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    username:  String,
    script:    String,
) {
    run_script_inner(req_tx, reply_rx, event_rx, items, state, stop_flag, username, script);
}

fn run_script_inner(
    req_tx:    crossbeam_channel::Sender<crate::script_channel::ScriptRequest>,
    reply_rx:  crossbeam_channel::Receiver<crate::script_channel::ScriptReply>,
    event_rx:  crossbeam_channel::Receiver<crate::bot::BotEventRaw>,
    items:     std::sync::Arc<crate::items::ItemsDat>,
    state:     std::sync::Arc<std::sync::RwLock<crate::bot_state::BotState>>,
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    username:  String,
    script:    String,
) {
    let lua = Lua::new_with(
        mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH | mlua::StdLib::IO,
        mlua::LuaOptions::default(),
    ).expect("lua init failed");

    let stop_hook = stop_flag.clone();
    let _ = lua.set_hook(
        mlua::HookTriggers { every_nth_instruction: Some(200), ..Default::default() },
        move |_lua, _debug| {
            if stop_hook.load(Ordering::Relaxed) {
                Err(LuaError::runtime("__script_stop__"))
            } else {
                Ok(mlua::VmState::Continue)
            }
        },
    );

    let proxy = BotProxy { req_tx, reply_rx, state: state.clone() };

    let setup = || -> LuaResult<()> {
        lua.globals().set("__bot", lua.create_userdata(proxy)?)?;
        lua.load("function getBot() return __bot end").exec()?;

        // ── sleep(ms) ─────────────────────────────────────────────────────────
        {
            let stop = stop_flag.clone();
            lua.globals().set("sleep", lua.create_function(move |_, ms: u64| {
                let deadline = std::time::Instant::now() + Duration::from_millis(ms);
                while std::time::Instant::now() < deadline {
                    if stop.load(Ordering::Relaxed) {
                        return Err(LuaError::runtime("__script_stop__"));
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(())
            })?)?;
        }

        // ── getInfo(id | name) ────────────────────────────────────────────────
        {
            let items2 = items.clone();
            lua.globals().set("getInfo", lua.create_function(move |_, val: LuaValue| {
                let info = match &val {
                    LuaValue::Integer(n) => items2.find_by_id(*n as u32).cloned(),
                    LuaValue::Number(n)  => items2.find_by_id(*n as u32).cloned(),
                    LuaValue::String(s)  => {
                        let raw = s.to_str().map(|b| b.to_string()).unwrap_or_default();
                        // try numeric first, then name search
                        if let Ok(id) = raw.parse::<u32>() {
                            items2.find_by_id(id).cloned()
                        } else {
                            items2.find_by_name(&raw).cloned()
                        }
                    }
                    _ => None,
                };
                Ok(info.map(LuaItemInfo))
            })?)?;
        }

        // ── getInfos() ────────────────────────────────────────────────────────
        {
            let items2 = items.clone();
            lua.globals().set("getInfos", lua.create_function(move |lua, ()| {
                let t = lua.create_table()?;
                for (i, item) in items2.items.iter().enumerate() {
                    t.set(i + 1, LuaItemInfo(item.clone()))?;
                }
                Ok(t)
            })?)?;
        }

        // ── read / write / append ─────────────────────────────────────────────
        lua.globals().set("read", lua.create_function(|_, path: String| {
            std::fs::read_to_string(&path)
                .map_err(|e| LuaError::runtime(e.to_string()))
        })?)?;

        lua.globals().set("write", lua.create_function(|_, (path, content): (String, String)| {
            std::fs::write(&path, content)
                .map_err(|e| LuaError::runtime(e.to_string()))
        })?)?;

        lua.globals().set("append", lua.create_function(|_, (path, content): (String, String)| {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .create(true).append(true).open(&path)
                .map_err(|e| LuaError::runtime(e.to_string()))?;
            f.write_all(content.as_bytes())
                .map_err(|e| LuaError::runtime(e.to_string()))
        })?)?;

        // ── removeColor(text) ─────────────────────────────────────────────────
        lua.globals().set("removeColor", lua.create_function(|_, text: String| {
            let mut out = String::with_capacity(text.len());
            let mut chars = text.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '`' {
                    // consume the next character (the color code) and skip both
                    chars.next();
                } else {
                    out.push(c);
                }
            }
            Ok(out)
        })?)?;

        // ── clearConsole() ────────────────────────────────────────────────────
        {
            let state2 = state.clone();
            lua.globals().set("clearConsole", lua.create_function(move |_, ()| {
                state2.write().unwrap().console.clear();
                Ok(())
            })?)?;
        }

        // ── getUsername() ─────────────────────────────────────────────────────
        {
            let uname = username.clone();
            lua.globals().set("getUsername", lua.create_function(move |_, ()| {
                Ok(uname.clone())
            })?)?;
        }

        // ── HttpClient.new() ─────────────────────────────────────────────────
        register_http_client(&lua)?;

        // ── Webhook.new() ─────────────────────────────────────────────────────
        register_webhook(&lua)?;

        // ── Event helpers (pure Lua) ──────────────────────────────────────────
        lua.load(r#"
__events = {}
__listening_stop = false

Event = { variantlist = 1, gameupdate = 2, gamemessage = 3 }

function addEvent(etype, fn)
    __events[etype] = fn
end
function removeEvent(etype)
    __events[etype] = nil
end
function removeEvents()
    __events = {}
end
function unlistenEvents()
    __listening_stop = true
end
        "#).exec()?;

        {
            let event_rx2 = event_rx.clone();
            let stop2     = stop_flag.clone();
            lua.globals().set("listenEvents", lua.create_function(move |lua, secs: Option<f64>| {
                let start = std::time::Instant::now();
                lua.globals().set("__listening_stop", false)?;

                while secs.map_or(true, |s| start.elapsed() < Duration::from_secs_f64(s)) {
                    if lua.globals().get::<bool>("__listening_stop").unwrap_or(false)
                        || stop2.load(Ordering::Relaxed)
                    {
                        break;
                    }

                    let handlers: LuaTable = lua.globals().get("__events")?;

                    while let Ok(event) = event_rx2.try_recv() {
                        match event {
                            BotEventRaw::VariantList { vl, net_id } => {
                                if let Ok(f) = handlers.get::<LuaFunction>(1u8) {
                                    let lua_vl = lua.create_userdata(LuaVariantList(vl))?;
                                    let _ = f.call::<()>((lua_vl, net_id));
                                }
                            }
                            BotEventRaw::GameUpdate { pkt } => {
                                if let Ok(f) = handlers.get::<LuaFunction>(2u8) {
                                    let lua_pkt = lua.create_userdata(LuaGameUpdatePacket(pkt))?;
                                    let _ = f.call::<()>(lua_pkt);
                                }
                            }
                            BotEventRaw::GameMessage { text } => {
                                if let Ok(f) = handlers.get::<LuaFunction>(3u8) {
                                    let _ = f.call::<()>(text);
                                }
                            }
                        }
                    }

                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(())
            })?)?;
        }

        // ── GameUpdatePacket.new() ─────────────────────────────────────────────
        let gup = lua.create_table()?;
        gup.set("new", lua.create_function(|_, ()| {
            Ok(LuaGameUpdatePacket(GameUpdatePacket::default()))
        })?)?;
        lua.globals().set("GameUpdatePacket", gup)?;

        // ── Shortcut globals ───────────────────────────────────────────────────
        lua.load(r#"
local __builtin_ipairs = ipairs
local __builtin_pairs = pairs

function ipairs(t)
    if type(t) == "table" and rawget(t, "__mori_sequence") then
        local function iter(seq, i)
            i = i + 1
            local item = seq[i]
            if item ~= nil then
                return i, item
            end
        end
        return iter, t, 0
    end
    return __builtin_ipairs(t)
end

function pairs(t)
    if type(t) == "table" and rawget(t, "__mori_sequence") then
        local function iter(seq, i)
            i = i + 1
            local item = seq[i]
            if item ~= nil then
                return i, item
            end
        end
        return iter, t, 0
    end
    return __builtin_pairs(t)
end

function getLocal()
    return getBot():getLocal()
end
function getWorld()
    return getBot():getWorld()
end
function getInventory()
    return getBot():getInventory()
end
function getPlayer(key)
    local w = getBot():getWorld()
    if w then return w:getPlayer(key) end
    return nil
end
function getPlayers()
    local w = getBot():getWorld()
    if w then return w:getPlayers() end
    return {}
end
function getTile(x, y)
    local w = getBot():getWorld()
    if w then return w:getTile(x, y) end
    return nil
end
function getTiles()
    local w = getBot():getWorld()
    if w then return w:getTiles() end
    return {}
end
function getObject(oid)
    local w = getBot():getWorld()
    if w then return w:getObject(oid) end
    return nil
end
function getObjects()
    local w = getBot():getWorld()
    if w then return w:getObjects() end
    return {}
end
function getNPC(id)
    local w = getBot():getWorld()
    if w then return w:getNPC(id) end
    return nil
end
function getNPCs()
    local w = getBot():getWorld()
    if w then return w:getNPCs() end
    return {}
end
function hasAccess(x, y)
    local w = getBot():getWorld()
    if w then return w:hasAccess(x, y) end
    return false
end
        "#).exec()?;

        Ok(())
    };

    if let Err(e) = setup() {
        state.write().unwrap().console.push(format!("`4[Lua setup error] {e}"));
        return;
    }

    if let Err(e) = lua.load(&script).exec() {
        if !e.to_string().contains("__script_stop__") {
            state.write().unwrap().console.push(format!("`4[Lua] {e}"));
        }
    }
}

