use std::sync::atomic::{AtomicBool, Ordering};
use crate::server::DashboardLinks;
use crate::types::login_info::LoginInfo;
use crate::types::player::Player;
use crate::types::server_data::ServerData;
use crate::Bot;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use mlua::{Lua, UserData, UserDataMethods};

#[derive(Debug, Default)]
pub struct State {
    pub hack_type: u32,
    pub build_length: u8,
    pub punch_length: u8,
    pub velocity: f32,
    pub gravity: f32,
}

#[derive(Debug, Default)]
pub struct DelayConfig {
    pub findpath_delay: u32,
    pub punch_delay: u32,
    pub place_delay: u32,
}

#[derive(Debug, Default)]
pub struct Automation {
    pub auto_collect: bool,
    pub auto_reconnect: bool,
}

pub struct Info {
    pub payload: Vec<String>,
    pub login_method: ELoginMethod,
    pub login_info: Mutex<Option<LoginInfo>>,
    pub server_data: Mutex<Option<ServerData>>,
    pub dashboard_links: Mutex<Option<DashboardLinks>>,
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

#[derive(Debug)]
pub struct World {
    pub data: Mutex<gtworld_r::World>,
    pub players: Mutex<HashMap<u32, Player>>,
}

impl Default for World {
    fn default() -> Self {
        World {
            data: Mutex::new(gtworld_r::World::new()),
            players: Mutex::new(HashMap::<u32, Player>::new()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ELoginMethod {
    APPLE,
    GOOGLE,
    LEGACY,
}

impl Default for ELoginMethod {
    fn default() -> Self {
        ELoginMethod::LEGACY
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

impl UserData for BotArc {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "sendPacket",
            |_, this, (packet_type, message): (u32, String)| {
                this.0.send_packet(packet_type.into(), message.as_bytes(), None, true);
                Ok(())
            },
        );
        methods.add_method("say", |_, this, message: String| {
            this.0.say(message);
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
                this.0.place(offset_x, offset_y, item_id);
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

    }

    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("pos", |_, this| {
            let pos = this.0.position.read().unwrap();
            Ok(Position(pos.0, pos.1))
        });
        fields.add_field_method_get("gems", |_, this| {
            let gems = this.0.gems.load(Ordering::SeqCst);
            Ok(gems)
        });
    }
}