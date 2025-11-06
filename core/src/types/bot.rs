use crate::Bot;
use mlua::{Lua, UserData, UserDataMethods, UserDataFields};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct State {
    pub hack_type: u32,
    pub build_length: u8,
    pub punch_length: u8,
    pub velocity: f32,
    pub gravity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct DelayConfig {
    pub findpath_delay: u32,
    pub punch_delay: u32,
    pub place_delay: u32,
}

impl Default for DelayConfig {
    fn default() -> Self {
        Self {
            findpath_delay: 150,
            punch_delay: 100,
            place_delay: 100,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Automation {
    pub auto_collect: bool,
    pub auto_reconnect: bool,
}

impl Default for Automation {
    fn default() -> Self {
        Self {
            auto_collect: true,
            auto_reconnect: true,
        }
    }
}

pub struct Scripting {
    pub data: Mutex<String>,
    pub currently_executing: AtomicBool,
    pub lua: Lua,
}

impl Default for Scripting {
    fn default() -> Self {
        Scripting {
            data: Mutex::new(String::new()),
            currently_executing: AtomicBool::new(false),
            lua: Lua::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LoginVia {
    GOOGLE,
    APPLE,
    LTOKEN([String; 4]),
    LEGACY([String; 2]),
}

impl Default for LoginVia {
    fn default() -> Self {
        LoginVia::LEGACY([String::new(), String::new()])
    }
}

#[derive(Default)]
pub struct TemporaryData {
    pub drop: Mutex<(u32, u32)>,
    pub trash: Mutex<(u32, u32)>,
    pub dialog_callback: Mutex<Option<fn(&Bot)>>,
}

#[derive(Clone)]
pub struct BotArc(pub Arc<Bot>);

pub struct Position(pub f32, pub f32);

impl UserData for Position {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("x", |_, this, ()| Ok(this.0));
        methods.add_method("y", |_, this, ()| Ok(this.1));
    }
}

// Lua-exposed World data snapshot
#[derive(Clone)]
pub struct WorldData {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<TileData>,
    pub floating: Vec<DroppedItemData>,
}

#[derive(Clone)]
pub struct TileData {
    pub fg: u16,
    pub bg: u16,
}

#[derive(Clone)]
pub struct DroppedItemData {
    pub id: u16,
    pub x: f32,
    pub y: f32,
    pub count: u8,
    pub uid: u32,
}

impl UserData for WorldData {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("width", |_, this| Ok(this.width));
        fields.add_field_method_get("height", |_, this| Ok(this.height));
        fields.add_field_method_get("tiles", |lua, this| {
            let table = lua.create_table()?;
            for (i, tile) in this.tiles.iter().enumerate() {
                table.set(i + 1, tile.clone())?;
            }
            Ok(table)
        });
        fields.add_field_method_get("floating", |lua, this| {
            let table = lua.create_table()?;
            for (i, item) in this.floating.iter().enumerate() {
                table.set(i + 1, item.clone())?;
            }
            Ok(table)
        });
    }
}

impl UserData for TileData {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("fg", |_, this| Ok(this.fg));
        fields.add_field_method_get("bg", |_, this| Ok(this.bg));
    }
}

impl UserData for DroppedItemData {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.id));
        fields.add_field_method_get("x", |_, this| Ok(this.x));
        fields.add_field_method_get("y", |_, this| Ok(this.y));
        fields.add_field_method_get("count", |_, this| Ok(this.count));
        fields.add_field_method_get("uid", |_, this| Ok(this.uid));
    }
}

// Lua-exposed Inventory data snapshot
#[derive(Clone)]
pub struct InventoryData {
    pub size: u32,
    pub items: Vec<ItemData>,
    pub gems: i32,
}

#[derive(Clone)]
pub struct ItemData {
    pub id: u16,
    pub amount: u8,
}

impl UserData for InventoryData {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
        fields.add_field_method_get("gems", |_, this| Ok(this.gems));
        fields.add_field_method_get("items", |lua, this| {
            let table = lua.create_table()?;
            for (i, item) in this.items.iter().enumerate() {
                table.set(i + 1, item.clone())?;
            }
            Ok(table)
        });
    }
}

impl UserData for ItemData {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.id));
        fields.add_field_method_get("amount", |_, this| Ok(this.amount));
    }
}

impl UserData for BotArc {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "sendPacket",
            |_, this, (packet_type, message): (u32, String)| {
                this.0
                    .send_packet(packet_type.into(), message.as_bytes(), None, true);
                Ok(())
            },
        );
        methods.add_method("say", |_, this, message: String| {
            this.0.say(&message);
            Ok(())
        });
        methods.add_method("disconnect", |_, this, ()| {
            this.0.disconnect();
            Ok(())
        });
        methods.add_method("warp", |_, this, world_name: String| {
            this.0.warp(world_name);
            Ok(())
        });
        methods.add_method("punch", |_, this, (offset_x, offset_y): (i32, i32)| {
            this.0.punch(offset_x, offset_y);
            Ok(())
        });
        methods.add_method("findPath", |_, this, (x, y): (u32, u32)| {
            this.0.find_path(x, y);
            Ok(())
        });
        methods.add_method(
            "place",
            |_, this, (offset_x, offset_y, item_id): (i32, i32, u32)| {
                this.0.place(offset_x, offset_y, item_id, false);
                Ok(())
            },
        );
        methods.add_method("wrench", |_, this, (offset_x, offset_y): (i32, i32)| {
            this.0.wrench(offset_x, offset_y);
            Ok(())
        });
        methods.add_method("wear", |_, this, item_id: u32| {
            this.0.wear(item_id);
            Ok(())
        });
        methods.add_method("walk", |_, this, (offset_x, offset_y): (i32, i32)| {
            this.0.walk(offset_x, offset_y, false);
            Ok(())
        });
        methods.add_method("autoCollect", |_, this, enabled: bool| {
            this.0.set_auto_collect(enabled);
            Ok(())
        });
        methods.add_method("autoReconnect", |_, this, enabled: bool| {
            this.0.set_auto_reconnect(enabled);
            Ok(())
        });
        methods.add_method("setFindPathDelay", |_, this, delay: u32| {
            this.0.set_findpath_delay(delay);
            Ok(())
        });
        methods.add_method("setPunchDelay", |_, this, delay: u32| {
            this.0.set_punch_delay(delay);
            Ok(())
        });
        methods.add_method("setPlaceDelay", |_, this, delay: u32| {
            this.0.set_place_delay(delay);
            Ok(())
        });
        methods.add_method("drop", |_, this, (item_id, amount): (u32, u32)| {
            this.0.drop_item(item_id, amount);
            Ok(())
        });
        methods.add_method("trash", |_, this, (item_id, amount): (u32, u32)| {
            this.0.trash_item(item_id, amount);
            Ok(())
        });
        methods.add_method("acceptAccess", |_, this, ()| {
            this.0.accept_access();
            Ok(())
        });
        methods.add_method("collect", |_, this, ()| {
            let collected = this.0.collect();
            Ok(collected)
        });
        methods.add_method("world", |_, this, ()| {
            // Lock the world and create a snapshot
            let world_lock = match this.0.world.data.try_lock() {
                Ok(w) => w,
                Err(_) => return Err(mlua::Error::runtime("Failed to lock world data")),
            };

            // Create tile data snapshot
            let tiles: Vec<TileData> = world_lock
                .tiles
                .iter()
                .map(|tile| TileData {
                    fg: tile.foreground_item_id,
                    bg: tile.background_item_id,
                })
                .collect();

            // Create dropped item data snapshot
            let floating: Vec<DroppedItemData> = world_lock
                .dropped
                .items
                .iter()
                .map(|item| DroppedItemData {
                    id: item.id,
                    x: item.x,
                    y: item.y,
                    count: item.count,
                    uid: item.uid,
                })
                .collect();

            Ok(WorldData {
                name: world_lock.name.clone(),
                width: world_lock.width,
                height: world_lock.height,
                tiles,
                floating,
            })
        });
        methods.add_method("inventory", |_, this, ()| {
            // Get inventory snapshot
            let inv_snapshot = match this.0.inventory.try_get_snapshot() {
                Some(snapshot) => snapshot,
                None => return Err(mlua::Error::runtime("Failed to lock inventory data")),
            };

            // Create items vector from snapshot
            let items: Vec<ItemData> = inv_snapshot
                .item_amounts
                .iter()
                .map(|(id, amount)| ItemData {
                    id: *id,
                    amount: *amount,
                })
                .collect();

            Ok(InventoryData {
                size: inv_snapshot.size,
                items,
                gems: this.0.inventory.gems(),
            })
        });
    }

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("pos", |_, this| {
            let pos = this.0.movement.position();
            Ok(Position(pos.0, pos.1))
        });
        fields.add_field_method_get("gems", |_, this| Ok(this.0.inventory.gems()));
    }
}
