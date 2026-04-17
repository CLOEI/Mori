use mlua::prelude::*;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::bot::BotEventRaw;
use crate::protocol::packet::{GamePacketType, GameUpdatePacket, PacketFlags};
use crate::player::Player;
use crate::script_channel::{ScriptRequest as Req, ScriptReply as Rep};
use crate::protocol::variant::Variant;
use crate::world::{TileFlags, TileType};

use super::http::register_http_client;
use super::webhook::register_webhook;
use super::types::{
    BotProxy, LuaGameUpdatePacket, LuaInventory, LuaInventoryItem, LuaItemInfo,
    LuaLogin, LuaNetObject, LuaNpc, LuaPlayer, LuaTile, LuaVariant, LuaVariantList, LuaWorld,
};

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
        methods.add_method("buy", |_, p, pack_id: String| {
            p.request(Req::Buy { item: pack_id });
            Ok(())
        });

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
            let t = lua.create_table()?;
            for (i, tile) in w.world.tile_map.tiles.iter().enumerate() {
                t.set(i + 1, LuaTile(tile.clone()))?;
            }
            Ok(t)
        });
        fields.add_field_method_get("objects", |lua, w| {
            let t = lua.create_table()?;
            for (i, obj) in w.world.objects.iter().enumerate() {
                t.set(i + 1, LuaNetObject(obj.clone()))?;
            }
            Ok(t)
        });
        fields.add_field_method_get("players", |lua, w| {
            let t = lua.create_table()?;
            for (i, p) in w.players.iter().enumerate() {
                t.set(i + 1, LuaPlayer(p.clone()))?;
            }
            Ok(t)
        });
        fields.add_field_method_get("npcs", |lua, w| {
            let t = lua.create_table()?;
            for (i, n) in w.world.npcs.iter().enumerate() {
                t.set(i + 1, LuaNpc(n.clone()))?;
            }
            Ok(t)
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getTile", |_, w, (x, y): (u32, u32)| {
            Ok(w.world.get_tile(x, y).cloned().map(LuaTile))
        });

        methods.add_method("getTiles", |lua, w, ()| {
            let t = lua.create_table()?;
            for (i, tile) in w.world.tile_map.tiles.iter().enumerate() {
                t.set(i + 1, LuaTile(tile.clone()))?;
            }
            Ok(t)
        });

        methods.add_method("getObject", |_, w, oid: u32| {
            Ok(w.world.objects.iter().find(|o| o.uid == oid).cloned().map(LuaNetObject))
        });

        methods.add_method("getObjects", |lua, w, ()| {
            let t = lua.create_table()?;
            for (i, obj) in w.world.objects.iter().enumerate() {
                t.set(i + 1, LuaNetObject(obj.clone()))?;
            }
            Ok(t)
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
            let t = lua.create_table()?;
            for (i, p) in w.players.iter().enumerate() {
                t.set(i + 1, LuaPlayer(p.clone()))?;
            }
            Ok(t)
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
            let t = lua.create_table()?;
            for (i, n) in w.world.npcs.iter().enumerate() {
                t.set(i + 1, LuaNpc(n.clone()))?;
            }
            Ok(t)
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

        lua.load(r#"
packs = [[
--Name:PackID:Gems:Items
UpgradeBackpack:upgrade_backpack::
ClothesPack:clothes_pack:50:234,210,1846,342,11202,214,138,354,2938,144,272,208,90,38,66,238,122,36,34,44,48,76,140,10034,694,142,236,270,130,40,68,42,374,372,3578,370
RareClothesPack:rare_clothes:500:11204,600,1762,1028,1146,292,1708,5016,576,1736,348,1652,1650,294,3052,1330,890,1124,312,3530,1140,156,2882,258,452,9356,154,1478,1142,1120,150,2934,80,2928,1524,796,1126,152,1954,2576,2958,158,136,496,1848,896,1476,4954,1864,6674,296,290,310,1862,6796,4964,4968,2700,1316,1516,1854,1856,446,3590,1312,1688,1314,3466,594,1858,8884,1502,268,794,1342,488,798,1590,724,352,3048,1026,1860,1032,1838,1718,1850,316,1690,492,894,6788,1116,304,1332,302,86,3164,2578,1022,2960,88,1800,3056,3060,3934,2954,132,852,266,494,1504,74,50,134,98,448,3932,96,148,788,232,288,4966,70,2718,1844,1722,84,212,1122,92,1720,124,46,72,356,146,754,662,94,78,82,1716,250
InfernalShades:infernal_shades:25000:12474
TidalShades:tidal_shades:25000:12476
VerdantShades:verdant_shades:25000:12478
Transmutabooth:transmutation_device:25000:9170
ContactLensPack:contact_lenses:15000:6768,6760,6762,6764,6766
EyeDropPack:eye_drops:30000:5468,5466,5464,5462,5460,5458
TurtleHat:nyan_hat:25000:574
TinyHorsie:tiny_horsie:25000:592
PleiadianStarShip:star_ship:25000:760
DragonHand:dragon_hand:50000:900
LittleRedCorvette:corvette:25000:766
StickHorse:stick_horse:25000:1012
Ambulance:ambulance:25000:1272
RidingRaptor:raptor:25000:1320
Mid-PacificOwl:owl:30000:1540
UnicornGarland:unicorn:50000:1648
StarBoard:starboard:30000:1740
GrowleyMotorcycle:motorcycle:50000:1950
MonkeyOnYourBack:monkey_on_back:50000:2900
CarrotSword:carrot_sword:15000:2908
RedBicycle:red_bicycle:30000:2974
FireTruck:fire_truck:50000:3068
PetSlime:pet_slime:100000:3166
DabstepLowTopSneakers:dabstep_shoes:30000:6790
WorldLock:world_lock:2000:242
WorldLockPack:world_lock_10_pack:20000:242
AdvertisersPack:anzu_pack1:10000:10874,10872,10870,10876
SmallLock:small_lock:50:202
BigLock:big_lock:200:204
HugeLock:huge_lock:500:206
DoorAndSignHelloPack:door_pack:12:12,20
DoorMover:door_mover:5000:1404
VendingMachine:vending_machine:8000:2978
DigiVendMachine:digi_vend:12000:9268
VendingHub-CheckoutCounter:checkout_counter:50000:9270
ChangeofAddress:change_addr:20000:2580
SignalJammer:signal_jammer:2000:226
PunchJammer:punch_jammer:15000:1276
ZombieJammer:zombie_jammer:15000:1278
ImperialStarshipBlast:starship_blast:10000:6420
SurgWorldBlast:surg_blast:10000:8556
BountifulBlast:bountiful_blast:5000:8738
ThermonuclearBlast:thermo_blast:15000:1402
AntigravityGenerator:antigravity_generator:450000:4992
BuildingBlocksMachine:building_blocks_machine:8000:8196
Builder'sLock:builders_lock:50000:4994
WeatherMachine-Sunny:weather_sunny:1000:932
WeatherMachine-Night:weather_night:10000:934
WeatherMachine-Arid:weather_arid:10000:946
WeatherMachine-RainyCity:weather_rainy:10000:984
WeatherMachine-WarpSpeed:weather_warp:10000:1750
MarsBlast:mars_blast:15000:1136
UnderseaBlast:undersea_blast:15000:1532
CaveBlast:cave_blast:30000:3562
WeatherMachine-Stuff:weather_stuff:50000:3832
WeatherMachine-Jungle:weather_jungle:20000:4776
WeatherMachine-Background:weather_backgd:150000:5000
WeatherMachine-DigitalRain:digital_rain_weather:30000:6854
TreasureBlast:treasure_blast:10000:7588
InfinityWeatherMachine:infinity_weather_machine:50000:10058
SmallSeedPack:5seed:100:5706
SmallSeedPackCollection:ssp_10_pack:1000:5706
RareSeedPack:rare_seed:1000:2293,2295,2297,2299,2301,2305,2629,2309,2311,2313,2315,2317,2321,2323,2325,2327,2329,2333,2335,2337,2339,2341,2319,2331,2343,4697,2447,2519,2629,2727,231,487,483,2019,427,1757,2009,281,781,329,459,1041,421,2787,2789,1043,2797,181,613,65,4983,283,553,425,4635,987,627,2017,989,61,4639,973,3523,331,265,859,433,253,339,1105,439,4567,333,677,5673,1427,5603,183,10533,5731,7223,10833,6035,6813,1113,1047,653,2811,4703,1429,847,665,485,1613,833,2025,1775,3931,429,3831,423,437,1773,4767,1299,361,667,2815,3473,301,4637,5683,2709,415,955,461,861,9389,5467,2791,2815,1803,419,2027,685,555,741,10531,5729,7221,10831,6033,6811,621,6401,1107,359,3471,855,309,1685,1449,1309,441,1111,221,1431,3065,9307,1257,1501,1109,4641,3587,1103,417,743,431,3823,2949,1131,3825,727,5379,729,187,787,5459,3423,3261,3521,5461,3721,327,247,63,3479,445,4783,299,863,8935,1451,261,1871,1499,455,9391,683,1159,551,1745,411,3005,557,687,4799,559,4643,3899,1605,9309,567,565,1155,3827,1327,1767,993,1157,563,4721,115,1423,10515,1329,6545,287,669,777,5463,1655,1599,561,4585,3413,4745,1771,2011,1161,1729,3757,3753,321,3265,3263,623,775,6397,129,4667,759,3767,1529,6543,4707,3835,1435,651,8929,569,527,3287,1595,6063,2795,1003,1731,4943,4963,597,4669,1531,1523,625,4297,599,657,571,873,3755,2969,1725,4603,1447,3267,5375,1683
5-packofGrowSprayFertilizer:grow_spray:200:228
DeluxeGrowSpray:deluxe_grow_spray:900:1778,5746,5742,5748
MutantPack:gross_mixers:10000:5746,5742,5748,8392,8394,8396,8398,8390
BountifulSeedPack:bountiful_seed_pack:1000:8970
BasicSplicingKit:basic_splice:200:101,21,1139,381,17,57,13,3567,11
SurgeryStarterPack:surg_starter_pack:5000:1248,1246,1244,1242,8558,1256,1240
SurgicalKit:surgical_kit:12000:9310,4316,4310,1270,1258,1268,1260,4308,4318,4312,4314,1264,1266,1263,4296
SurgicalToolsValuePack:surg_value_pack:45000:9310,4316,4310,1270,1258,1268,1260,4308,4318,4312,4314,1264,1266,1262,4296
GalacticGoodies:star_supplies:15000:6526,6532,6534,6536,6416,6524,6530,6518,6540,6528,6522,6538,6520
Fishin'Pack:fishin_pack:10000:2912,3044,3004,3002,2914,5524,5522
FishTrainin'Pack:fish_training_pack:10000:5536,5532,5530
FishFlakes:fish_flakes:7500:5536
FishMedicine:fish_medicine:1500:5532
FishReviver:fish_reviver:5000:5534
crimewave:crime_wave:5000:2382,2380,2322,2326,2328,2320,2324,2336,2340,2338,2332,2334,2296,2292,2294,2298,2300,2314,2312,2310,2308,2316
Silkworm:silkworn:7000:3316
GeigerCounter:geiger:25000:2204
ZombieDefensePack:zombie_pack:10000:764,768,784,nul,nul,782,786,774,788,790
B.O.O.TrainingPack:boo_pack:10000:3710,3716,3714,3720
EctoJuicer:ectojuicer:30000:6096
GhostHunter'sPack:ghost_hunting:10000:13820
nian_lunar_chest:nian_lunar_chest:25000:
ChemsynthPack:chemsynth:10000:4442,4434,4440,4438,4436,4432,4430
RacingActionPack:race_pack:3500:486,484,482,410,430,428,496,356,488,490,494,492
Composer'sPack:music_pack:5000:418,416,414,426,424,422,420,412
EducationPack:school_pack:5000:678,672,670,676,658,654,650,652,674,8872,8870,8876,8874
DungeonPack:dungeon_pack:10000:692,696,694,690,688,686,684,682,680
FantasyPack:fantasy_pack:5000:583,596,598,606,608,604,594,600
VegasPack:vegas_pack:20000:798,796,794,758,756,754,456,752,1582,740,1922,1920,1924,1918,1916,743,747,749,744
FarmPack:farm_pack:15000:898,896,894,892,13732,890,886,888,884,880,872,866
MadScienceKit:science_pack:5000:918,920,924,914,916,930,904,12494,770,772,912,928
CityPack:city_pack:8000:1008,1002,988,1006,1004,998,996,990,992,986,994
WildWestPack:west_pack:8000:1016,1020,1048,1046,1044,1042,1040,1038,1032,1034,1036,1028,1026,1024,1030,1022
AstroPack:astro_pack:5000:1152,1154,1130,1128,1160,1162,1164,1146,1144,1142,1140,1148
PrehistoricPack:prehistoric_pack:5000:1328,1324,1322,1326,1342,1340,1332,1334,1338,1330,1336
ShopPack:shop_pack:10000:1420,1434,1446,1430,1432,1428,1426,1422,1436
HomePack:home_pack:50000:1474,1476,1478,1454,1472,1450,1480,1448,1482
CinemaPack:cinema_pack:6000:1596,1608,1606,1604,1600,1598,1594,1590,1592,1588
AdventurePack:adventure_pack:20000:4780,1704,1692,1698,1696,1702,1694,4706,4704,1706,1684,1682
Rockin'Pack:rockin_pack:9999:1726,1730,1728,1724,1720,1722,1732,1718,1714,1712,1710
GamePack:game_pack:50000:1658,1656,1622,1618,1616,1660
SuperheroPack:superhero:10000:6684,7148,2196,2200,2202,2198,2194,6694,2162,2164,2160,2158,6696,6686,2126,2128,2124,2122,6692,6682,2102,2104,2100,2098,2114,2116,2112,2110,6690,6680,2174,2176,2172,2170,2078,2080,2076,2074,6688,6678,2138,2140,2136,2134,2186,2188,2184,2182,2150,2152,2148,2146,2208
FashionPack:fashion_pack:5000:11800,11206,10080,2672,2684,2660,2648,2708,2646
SportsballPack:sportsball_pack:20000:13600,6676,6670,2906,2888,2884,2880,6672,2882,2878,2890,2886,6668,2894,2892,2898,2896,6674,2904,2902
FirefighterPack:firefighter:10000:3072,3066,3052,3060,3056,3048
Steampack:steampack:20000:3292,3280,3284,3282,3270,3260
Painter'sPack:paintbrush:30000:3494,3492,3480,3484,3486,3488,3482,3478,3490
Paleontologist'sKit:paleo_kit:20000:3938,4128,3934,3932,4132
CyBlocksStarterPack:robot_starter_pack:5000:5668,5680,5676,5672,5678,5674
CyBlocksCommandPack:robot_command_pack:2000:5704,5702,5698,5700,5696,5694,5690,5692,5688,5686,5682,5684,5680,5678,5676,5672,5674
CyBotPack:robot_pack:15000:5670,5710,5708
GanglandStyle:gang_pack:5000:478,480,472,470,476,474,468,466,464,462
Kuwii'sTutelar:kuwiistutelar:250000:15612
]]

PackDB = {}
PackNames = {}

do
    for line in packs:gmatch("[^\r\n]+") do
        if line ~= "" and line:sub(1, 2) ~= "--" then
            local name, pack_id, gems, items = line:match("^([^:]+):([^:]+):([^:]*):(.*)$")
            if name and pack_id then
                local parsed_items = {}
                if items and items ~= "" then
                    for token in items:gmatch("[^,]+") do
                        token = token:match("^%s*(.-)%s*$")
                        parsed_items[#parsed_items + 1] = tonumber(token) or token
                    end
                end

                local entry = {
                    name = name,
                    pack_id = pack_id,
                    gems = tonumber(gems) or 0,
                    items = parsed_items,
                    items_raw = items or "",
                }

                PackDB[pack_id] = entry
                PackNames[name] = entry
            end
        end
    end
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
function getLocal()
    return getBot():getLocal()
end
function getWorld()
    return getBot():getWorld()
end
function getInventory()
    return getBot():getInventory()
end
function buy(pack_id)
    return getBot():buy(pack_id)
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

