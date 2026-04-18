use crate::{cursor::Cursor, world::constants::{BUILDERS_LOCK_TILE_ID, NON_WORLDLOCK_TILE_IDS}};
use anyhow::{Result, bail};

use super::constants::{CBOR_TILE_IDS, MAP_VERSION_MIN, MAX_TILE_COUNT, MAX_WORLD_OBJECTS};

// ── World ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct World {
    pub version: u16,
    pub flags: u32,
    pub tile_map: WorldTileMap,
    pub objects: Vec<WorldObject>,
    pub npcs: Vec<WorldNpc>,
    pub next_object_uid: u32,
    pub base_weather: u16,
    pub current_weather: u16,
}

impl World {
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        let idx = (y as usize).checked_mul(self.tile_map.width as usize)?.checked_add(x as usize)?;
        self.tile_map.tiles.get(idx)
    }

    pub fn set_npc(&mut self, npc: WorldNpc) {
        if let Some(existing) = self.npcs.iter_mut().find(|n| n.id == npc.id) {
            *existing = npc;
        } else {
            self.npcs.push(npc);
        }
    }

    pub fn remove_npc(&mut self, id: u8) {
        self.npcs.retain(|n| n.id != id);
    }

    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        let idx = (y as usize).checked_mul(self.tile_map.width as usize)?.checked_add(x as usize)?;
        self.tile_map.tiles.get_mut(idx)
    }

    /// Update a tile from a raw SendTileUpdateData blob.
    /// Layout: fg(u16) bg(u16) parent(u16) flags(u16) [kind(u8) extra...]
    /// Returns the new (fg, bg) on success.
    pub fn update_tile(&mut self, x: u32, y: u32, cur: &mut Cursor, map_version: u16) -> Result<(u16, u16)> {
        let width = self.tile_map.width;
        let height = self.tile_map.height;
        let target_tile = self.get_tile_mut(x, y)
            .ok_or_else(|| anyhow::anyhow!("target_tile.is_none! coord: {x},{y}, world size: {width},{height}"))?;

        let result = Tile::parse(cur, map_version, x, y)?;
        let fg = result.fg_item_id;
        let bg = result.bg_item_id;
        *target_tile = result;

        Ok((fg, bg))
    }

    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cur = Cursor::new(data, "world map blob");

        let version = cur.u16()?;
        if version < MAP_VERSION_MIN {
            bail!("map version {version:#x} < minimum {MAP_VERSION_MIN:#x}");
        }

        let flags = cur.u32()?;
        let tile_map = WorldTileMap::parse(&mut cur, version)?;
        let (objects, last_dropped_uid) = parse_world_objects(&mut cur)?;

        let base_weather = cur.u16()?;
        cur.skip(2)?;
        let current_weather = cur.u16()?;

        let next_object_uid = last_dropped_uid + 1;

        Ok(World {
            version,
            flags,
            tile_map,
            objects,
            npcs: Vec::new(),
            next_object_uid,
            base_weather,
            current_weather,
        })
    }
}

// WorldTilePermission

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct WorldTilePermission: u8 {
        const NONE = 0;
        const BUILD = 1;
        const BREAK = 2;
        const FULL_ACCESS = 3;
    }
}

// ── WorldTileMap ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WorldTileMap {
    pub world_name: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    pub world_lock_index: Option<usize>,
}

impl WorldTileMap {
    fn parse(cur: &mut Cursor, map_version: u16) -> Result<Self> {
        let name_len = cur.u16()? as usize;
        let name_raw = cur.bytes(name_len)?;
        let world_name = String::from_utf8_lossy(&name_raw).into_owned();

        let mut world_lock_index: Option<usize> = None;
        let width = cur.u32()?;
        let height = cur.u32()?;
        let tile_count = cur.u32()?;

        cur.skip(5)?;

        if tile_count >= MAX_TILE_COUNT {
            bail!("tile_count {tile_count} >= limit {MAX_TILE_COUNT}");
        }

        let mut tiles = Vec::with_capacity(tile_count as usize);
        for idx in 0..tile_count {
            let x = idx % width;
            let y = idx / width;
            let pos_before = cur.pos();
            match Tile::parse(cur, map_version, x, y) {
                Ok(t) => {
                    if let TileType::Lock{ .. } = t.tile_type && !NON_WORLDLOCK_TILE_IDS.contains(&t.fg_item_id) {
                        world_lock_index = Some(idx as usize);
                    }
                    tiles.push(t);
                }
                Err(e) => {
                    eprintln!("[world] tile {idx} ({x},{y}) failed at pos {pos_before}: {e}");
                    return Err(e);
                }
            }
        }

        cur.skip(12)?;

        Ok(WorldTileMap {
            world_name,
            width,
            height,
            tiles,
            world_lock_index,
        })
    }

    // Retrieve the lock that owns that tile
    pub fn get_tile_parent(&self, x: u32, y: u32) -> Option<&Tile> {
        let idx = (y * self.width + x) as usize;
        if idx >= self.tiles.len() { return None; }

        let tile = self.tiles.get(idx).unwrap();
        if !tile.flags.contains(TileFlags::HAS_PARENT) { return None; }

        self.tiles.get(tile.parent_block as usize)
    }

    // return what action is allowed on that tile
    pub fn get_tile_permission(&self, x: u32, y: u32, user_id: u32) -> WorldTilePermission {
        let idx = (y * self.width + x) as usize;
        let Some(tile) = self.tiles.get(idx) else {
            return WorldTilePermission::NONE;
        };

        if let TileType::Lock { owner_uid, .. } = tile.tile_type {
            return if user_id == owner_uid { WorldTilePermission::FULL_ACCESS }
            else { WorldTilePermission::NONE };
        }

        let tile_parent = self.get_tile_parent(tile.x, tile.y);

        if let Some(world_lock_index) = self.world_lock_index {
            if let Some(world_lock) = self.tiles.get(world_lock_index) {
                let TileType::Lock { owner_uid, ref access_uids, .. } = world_lock.tile_type else {
                    eprintln!("[world] world_lock tile type is not TileType::Lock. {:?}", world_lock.tile_type);
                    return WorldTilePermission::NONE;
                };

                if user_id == owner_uid {
                    return WorldTilePermission::FULL_ACCESS;
                }
                else if access_uids.contains(&user_id) && tile_parent.is_none() {
                    return WorldTilePermission::FULL_ACCESS;
                }
                else if world_lock.flags.contains(TileFlags::IS_OPEN_TO_PUBLIC) && tile_parent.is_none() {
                    return WorldTilePermission::FULL_ACCESS;
                }
            }
        }

        // Area lock. This can apply restrriction on certain area, overriding world lock
        // like Small lock, big lock, etc
        if let Some(lock) = tile_parent {
            let TileType::Lock { settings, owner_uid, ref access_uids, .. } = lock.tile_type else {
                eprintln!("[world] parent_tile type is not TileType::Lock. {:?}", lock.tile_type);
                return WorldTilePermission::NONE;
            };

            if user_id == owner_uid {
                return WorldTilePermission::FULL_ACCESS;
            }

            let flags = TileLockFlags::from_bits_retain(settings);
            if lock.fg_item_id == BUILDERS_LOCK_TILE_ID {
                // if is admin in builders lock
                if access_uids.contains(&user_id) || lock.flags.contains(TileFlags::IS_OPEN_TO_PUBLIC) {
                    if !flags.contains(TileLockFlags::ADMIN_LIMITED) {
                        return WorldTilePermission::FULL_ACCESS;
                    }

                    if flags.contains(TileLockFlags::BREAK_ONLY) { return WorldTilePermission::BREAK; }
                    else { return WorldTilePermission::BUILD; }
                }
            }
            else {
                if access_uids.contains(&user_id) || lock.flags.contains(TileFlags::IS_OPEN_TO_PUBLIC) {
                    return WorldTilePermission::FULL_ACCESS;
                }
            }
        }

        return WorldTilePermission::NONE;
    }
}

// ── TileFlags ─────────────────────────────────────────────────────────────────

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TileFlags: u16 {
        const HAS_EXTRA_DATA        = 0x0001;
        const HAS_PARENT            = 0x0002;
        const WAS_SPLICED           = 0x0004;
        const WILL_SPAWN_SEEDS_TOO  = 0x0008;
        const IS_SEEDLING           = 0x0010;
        const FLIPPED_X             = 0x0020;
        const IS_ON                 = 0x0040;
        const IS_OPEN_TO_PUBLIC     = 0x0080;
        const BG_IS_ON              = 0x0100;
        const FG_ALT_MODE           = 0x0200;
        const IS_WET                = 0x0400;
        const GLUED                 = 0x0800;
        const ON_FIRE               = 0x1000;
        const PAINTED_RED           = 0x2000;
        const PAINTED_GREEN         = 0x4000;
        const PAINTED_BLUE          = 0x8000;
    }
}

// ── Tile ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Tile {
    pub fg_item_id: u16,
    pub bg_item_id: u16,
    pub parent_block: u16,
    pub flags: TileFlags,
    pub flags_raw: u16,
    pub x: u32,
    pub y: u32,
    pub tile_type: TileType,
}

impl Tile {
    pub fn parse(cur: &mut Cursor, _map_version: u16, x: u32, y: u32) -> Result<Self> {
        let fg_item_id = cur.u16()?;
        let bg_item_id = cur.u16()?;
        let parent_block = cur.u16()?;
        let flags_raw = cur.u16()?;
        let flags = TileFlags::from_bits_retain(flags_raw);

        if flags.contains(TileFlags::HAS_PARENT) {
            cur.u16()?;
        }

        let mut tile_type = TileType::Basic;

        if flags.contains(TileFlags::HAS_EXTRA_DATA) {
            let kind = cur.u8()?;
            tile_type = parse_tile_extra(cur, kind, fg_item_id)?;
        }

        // CBOR blob for specific tile types (after TileExtraData)
        if CBOR_TILE_IDS.contains(&fg_item_id) {
            let cbor_size = cur.u32()? as usize;
            cur.skip(cbor_size)?; // skip raw CBOR bytes
        }

        Ok(Tile {
            fg_item_id,
            bg_item_id,
            parent_block,
            flags,
            flags_raw,
            x,
            y,
            tile_type,
        })
    }
}


bitflags::bitflags! {
    // TileLockFlags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TileLockFlags: u8 {
        const IGNORE_EMPTY_AIR     = 0x01;
        const DISABLE_MUSIC_NOTE   = 0x10;
        const INVISIBLE_MUSIC_NOTE = 0x20;
        const BREAK_ONLY           = 0x40;
        const ADMIN_LIMITED        = 0x80;
    }

    // put more flags here...
}



// ── TileType ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum TileType {
    Basic,
    Door {
        label: String,
        flags: u8,
    },
    Sign {
        label: String,
    },
    Lock {
        settings: u8,
        owner_uid: u32,
        access_count: u32,
        access_uids: Vec<u32>,
        bpm: i32,
        minimum_level: u32,
        world_timer: u32,
    },
    Seed {
        age: u32,
        item_on_tree: u8,
    },
    // Mailbox {
    //     s1: String,
    //     s2: String,
    //     s3: String,
    //     u4: u8,
    // },
    // Bulletin {
    //     s1: String,
    //     s2: String,
    //     s3: String,
    //     u4: u8,
    // },
    Dice {
        symbol: u8,
    },
    Provider {
        age: u32,
    },
    AchievementBlock {
        data: u32,
        tile_type: u8,
    },
    HeartMonitor {
        player_id: u32,
        player_name: String,
    },
    // DonationBox {
    //     s1: String,
    //     s2: String,
    //     s3: String,
    //     u4: u8,
    // },
    // BigLock {
    //     s1: String,
    //     s2: String,
    //     s3: String,
    //     u4: u8,
    // },
    Mannequin {
        label: String,
        unknown_1: u8,
        unknown_2: u16,
        unknown_3: u16,
        hat: u16,
        shirt: u16,
        pants: u16,
        boots: u16,
        face: u16,
        hand: u16,
        back: u16,
        hair: u16,
        neck: u16,
    },
    BunnyEgg {
        egg_placed: u32,
    },
    GameGrave {
        team: u8,
    },
    GameGenerator,
    XenoniteCrystal {
        unknown_1: u8,
        unknown_2: u32,
    },
    PhoneBooth {
        hat: u16,
        shirt: u16,
        pants: u16,
        shoes: u16,
        face: u16,
        hand: u16,
        back: u16,
        hair: u16,
        neck: u16,
    },
    Crystal {
        crystals: Vec<u8>,
    },
    CrimeInProgress {
        label: String,
        unknown_1: u32,
        unknown_2: u8,
    },
    Spotlight,
    DisplayBlock {
        item_id: u32,
    },
    VendingMachine {
        item_id: u32,
        price: i32,
    },
    FishTankPort {
        flags: u8,
        fishes: Vec<(u32, u32)>,
    },
    SolarCollector {
        data: [u8; 5],
    },
    Forge {
        temperature: u32,
    },
    GivingTree {
        harvested: u8,
        age: u16,
        unknown_1: u16,
        decoration_percentage: u8,
    },
    SteamOrgan {
        instrument_type: u8,
        note: u32,
    },
    SilkWorm {
        flags: u8,
        name: String,
        age: u32,
        unknown_1: u32,
        unknown_2: u32,
        can_be_fed: u8,
        food_saturation: u32,
        water_saturation: u32,
        color: u32,
        sick_duration: u32,
    },
    SewingMachine {
        bolt_ids: Vec<u32>,
    },
    CountryFlag {
        country: String,
    },
    LobsterTrap,
    PaintingEasel {
        item_id: u32,
        label: String,
    },
    PetBattleCage {
        label: String,
        base_pet: u32,
        pet_1: u32,
        pet_2: u32,
    },
    PetTrainer {
        label: String,
        pet_total_count: u32,
        unknown_1: u32,
        pets_id: Vec<u32>,
    },
    SteamEngine {
        temperature: u32,
    },
    LockBot {
        age: u32,
    },
    WeatherMachine {
        settings: u32,
    },
    SpiritStorageUnit {
        ghost_jar_count: u32,
    },
    DataBedrock,
    Shelf {
        top_left_item_id: u32,
        top_right_item_id: u32,
        bottom_left_item_id: u32,
        bottom_right_item_id: u32,
    },
    VipEntrance {
        unknown_1: u8,
        owner_uid: u32,
        access_uids: Vec<u32>,
    },
    ChallangeTimer,
    FishWallMount {
        label: String,
        item_id: u32,
        lb: u8,
    },
    Portrait {
        label: String,
        unknown_1: u32,
        unknown_2: u32,
        unknown_3: [u8; 5],
        unknown_4: u8,
        unknown_5: u16,
        face: u16,
        hat: u16,
        hair: u16,
        unknown_6: u32,
        infinity_crown_data: Option<String>,
    },
    WeatherMachine2 {
        unknown_1: u32,
        gravity: u32,
        flags: u8,
    },
    FossilPrepStation {
        unknown_1: u32,
    },
    DnaExtractor,
    Howler,
    ChemsynthTank {
        current_chem: u32,
        target_chem: u32,
    },
    StorageBlock {
        items: Vec<(u32, u32)>, // (id, amount)
    },
    CookingOven {
        temperature_level: u32,
        ingredients: Vec<(u32, u32)>, // (item_id, time_added)
        unknown_1: u32,
        unknown_2: u32,
        unknown_3: u32,
    },
    AudioRack {
        note: String,
        volume: u32,
    },
    GeigerCharger {
        unknown_1: u32,
    },
    AdventureBegins,
    TombRobber,
    BalloonOMatic {
        total_rarity: u32,
        team_type: u8,
    },
    TrainingPort {
        fish_lb: u32,
        fish_status: u16,
        fish_id: u32,
        fish_total_exp: u32,
        unknown_1: [u8; 8],
        fish_level: u32,
        unknown_2: u32,
        unknown_3: [u8; 5],
    },
    ItemSucker {
        item_id_to_suck: u32,
        item_amount: u32,
        flags: u16,
        limit: u32,
    },
    CyBot {
        command_datas: Vec<(u32, u32)>, // (command_id, is_command_used)
        sync_timer: u32,
        activated: u32,
    },
    // GuildBlock {
    //     label: String,
    // },
    GuildItem {
        unknown_1: [u8; 17],
    },
    Growscan {
        unknown_1: u8,
    },
    ContainmentFieldPowerNode {
        time: u32,
        linked_nodes: Vec<u32>,
    },
    SpiritBoard {
        player_required: u32,
        unk1: String,
        command: String,
        required_items: Vec<u32>,
    },
    TesseractManipulator {
        gems: u32,
        next_update_ms: u32,
        item_id: u32,
        enabled: u32,
    },
    StormyCloud {
        sting_duration: u32,
        is_solid: u32,
        non_solid_duration: u32,
    },
    TemporaryPlatform {
        unknown_1: u32,
    },
    SafeVault,
    AngelicCountingCloud {
        state: u32,
        unknown_1: u16,
        ascii_code: Option<u8>,
    },
    InfinityWeatherMachine {
        interval_minutes: u32,
        weather_machine_list: Vec<u32>,
    },
    PineappleGuzzler {
        pineapple_count: u32,
    },
    KrakenGalaticBlock {
        pattern_index: u8,
        unknown_1: u32,
        r: u8,
        g: u8,
        b: u8,
    },
    FriendsEntrance {
        owner_user_id: u32,
        allowed_friends_userid: Vec<u32>,
    },
    Unknown {
        kind: u8,
    },
}

// ── TileExtraData dispatch ────────────────────────────────────────────────────

fn parse_tile_extra(cur: &mut Cursor, kind: u8, fg_item_id: u16) -> Result<TileType> {
    match kind {
        1 => {
            // Door
            let label = cur.plain_string()?;
            let flags = cur.u8()?;
            Ok(TileType::Door { label, flags })
        }
        2 => {
            // Sign
            let label = cur.plain_string()?;
            cur.u32()?; // always 0xFFFFFFFF
            Ok(TileType::Sign { label })
        }
        3 => {
            // Lock
            let settings = cur.u8()?;
            let owner_uid = cur.u32()?;
            let access_count = cur.u32()?;
            let mut bpm: i32 = 100;
            let mut access_uids = Vec::with_capacity(access_count as usize);
            for _ in 0..access_count {
                let id = cur.i32()?;
                if id < 0 { bpm = id; }
                else { access_uids.push(id as u32); }
            }
            let minimum_level = cur.u32()?;
            let world_timer = cur.u32()?;

            // Guild Lock
            if fg_item_id == 5814 {
                cur.skip(16)?;
            }
            Ok(TileType::Lock {
                settings,
                owner_uid,
                access_count,
                access_uids,
                bpm,
                minimum_level,
                world_timer
            })
        }
        4 => {
            // Seed
            let age = cur.u32()?;
            let item_on_tree = cur.u8()?;
            Ok(TileType::Seed { age, item_on_tree })
        }
        // 6 => {
        //     // Mailbox
        //     let s1 = cur.plain_string()?;
        //     let s2 = cur.plain_string()?;
        //     let s3 = cur.plain_string()?;
        //     let u4 = cur.u8()?;
        //     Ok(TileType::Mailbox { s1, s2, s3, u4 })
        // }
        // 7 => {
        //     // Bulletin
        //     let s1 = cur.plain_string()?;
        //     let s2 = cur.plain_string()?;
        //     let s3 = cur.plain_string()?;
        //     let u4 = cur.u8()?;
        //     Ok(TileType::Bulletin { s1, s2, s3, u4 })
        // }
        8 => {
            // Dice
            let symbol = cur.u8()?;
            Ok(TileType::Dice { symbol })
        }
        9 => {
            // ChemicalSource
            let age = cur.u32()?;
            if fg_item_id == 10656 {
                cur.skip(4)?;
            }
            Ok(TileType::Provider { age })
        }
        10 => {
            // AchievementBlock
            let data = cur.u32()?;
            let tile_type = cur.u8()?;
            Ok(TileType::AchievementBlock { data, tile_type })
        }
        11 => {
            // HearthMonitor
            let player_id = cur.u32()?;
            let player_name = cur.plain_string()?;
            Ok(TileType::HeartMonitor {
                player_id,
                player_name,
            })
        }
        // 12 => {
        //     // DonationBox
        //     let s1 = cur.plain_string()?;
        //     let s2 = cur.plain_string()?;
        //     let s3 = cur.plain_string()?;
        //     let u4 = cur.u8()?;
        //     Ok(TileType::DonationBox { s1, s2, s3, u4 })
        // }
        // 13 => {
        //     // BigLock
        //     let s1 = cur.plain_string()?;
        //     let s2 = cur.plain_string()?;
        //     let s3 = cur.plain_string()?;
        //     let u4 = cur.u8()?;
        //     Ok(TileType::BigLock { s1, s2, s3, u4 })
        // }
        14 => {
            // Mannequin
            let label = cur.plain_string()?;
            let unknown_1 = cur.u8()?;
            let unknown_2 = cur.u16()?;
            let unknown_3 = cur.u16()?;
            let hat = cur.u16()?;
            let shirt = cur.u16()?;
            let pants = cur.u16()?;
            let boots = cur.u16()?;
            let face = cur.u16()?;
            let hand = cur.u16()?;
            let back = cur.u16()?;
            let hair = cur.u16()?;
            let neck = cur.u16()?;
            Ok(TileType::Mannequin {
                label,
                unknown_1,
                unknown_2,
                unknown_3,
                hat,
                shirt,
                pants,
                boots,
                face,
                hand,
                back,
                hair,
                neck,
            })
        }
        15 => {
            // BunnyEgg
            let egg_placed = cur.u32()?;
            Ok(TileType::BunnyEgg { egg_placed })
        }
        16 => {
            // GameGrave
            let team = cur.u8()?;
            Ok(TileType::GameGrave { team })
        }
        17 => Ok(TileType::GameGenerator),
        18 => {
            // XenoniteCrystal
            let unknown_1 = cur.u8()?;
            let unknown_2 = cur.u32()?;
            Ok(TileType::XenoniteCrystal {
                unknown_1,
                unknown_2,
            })
        }
        19 => {
            // PhoneBooth
            let hat = cur.u16()?;
            let shirt = cur.u16()?;
            let pants = cur.u16()?;
            let shoes = cur.u16()?;
            let face = cur.u16()?;
            let hand = cur.u16()?;
            let back = cur.u16()?;
            let hair = cur.u16()?;
            let neck = cur.u16()?;
            Ok(TileType::PhoneBooth {
                hat,
                shirt,
                pants,
                shoes,
                face,
                hand,
                back,
                hair,
                neck,
            })
        }
        20 => {
            // Crystal
            let crystal_count = cur.u16()?;
            let mut crystals = Vec::with_capacity(crystal_count as usize);
            for _ in 0..crystal_count {
                crystals.push(cur.u8()?);
            }
            Ok(TileType::Crystal { crystals })
        }
        21 => {
            // CrimeInProgress
            let label = cur.plain_string()?;
            let unknown_1 = cur.u32()?;
            let unknown_2 = cur.u8()?;
            Ok(TileType::CrimeInProgress {
                label,
                unknown_1,
                unknown_2,
            })
        }
        22 => Ok(TileType::Spotlight),
        23 => {
            // DisplayBlock
            let item_id = cur.u32()?;
            Ok(TileType::DisplayBlock { item_id })
        }
        24 => {
            // VendingMachine
            let item_id = cur.u32()?;
            let price = cur.i32()?;
            Ok(TileType::VendingMachine { item_id, price })
        }
        25 => {
            // FishTankPort
            let flags = cur.u8()?;
            let fish_count = cur.u32()?;
            let mut fishes = Vec::new();
            for _ in 0..(fish_count / 2) {
                let fish_item_id = cur.u32()?;
                let lbs = cur.u32()?;
                fishes.push((fish_item_id, lbs));
            }
            Ok(TileType::FishTankPort { flags, fishes })
        }
        26 => {
            // SolarCollector
            let mut data = [0u8; 5];
            for b in &mut data {
                *b = cur.u8()?;
            }
            Ok(TileType::SolarCollector { data })
        }
        27 => {
            // Forge
            let temperature = cur.u32()?;
            Ok(TileType::Forge { temperature })
        }
        28 => {
            // GivingTree
            let harvested = cur.u8()?;
            let age = cur.u16()?;
            let unknown_1 = cur.u16()?;
            let decoration_percentage = cur.u8()?;
            Ok(TileType::GivingTree {
                harvested,
                age,
                unknown_1,
                decoration_percentage,
            })
        }
        30 => {
            // SteamOrgan
            let instrument_type = cur.u8()?;
            let note = cur.u32()?;
            Ok(TileType::SteamOrgan {
                instrument_type,
                note,
            })
        }
        31 => {
            // SilkWorm
            let flags = cur.u8()?;
            let name = cur.plain_string()?;
            let age = cur.u32()?;
            let unknown_1 = cur.u32()?;
            let unknown_2 = cur.u32()?;
            let can_be_fed = cur.u8()?;
            let food_saturation = cur.u32()?;
            let water_saturation = cur.u32()?;
            let color = cur.u32()?;
            let sick_duration = cur.u32()?;
            Ok(TileType::SilkWorm {
                flags,
                name,
                age,
                unknown_1,
                unknown_2,
                can_be_fed,
                food_saturation,
                water_saturation,
                color,
                sick_duration,
            })
        }
        32 => {
            // SewingMachine
            let bolt_len = cur.u32()?;
            let mut bolt_ids = Vec::new();
            for _ in 0..bolt_len {
                bolt_ids.push(cur.u32()?);
            }
            Ok(TileType::SewingMachine { bolt_ids })
        }
        33 => {
            // CountryFlag
            let country = if fg_item_id == 3394 {
                cur.plain_string()?
            } else {
                String::new()
            };
            Ok(TileType::CountryFlag { country })
        }
        34 => Ok(TileType::LobsterTrap),
        35 => {
            // PaintingEasel
            let item_id = cur.u32()?;
            let label = cur.plain_string()?;
            Ok(TileType::PaintingEasel { item_id, label })
        }
        36 => {
            // PetBattleCage
            let label = cur.plain_string()?;
            let base_pet = cur.u32()?;
            let pet_1 = cur.u32()?;
            let pet_2 = cur.u32()?;
            Ok(TileType::PetBattleCage {
                label,
                base_pet,
                pet_1,
                pet_2,
            })
        }
        37 => {
            // PetTrainer
            let label = cur.plain_string()?;
            let pet_total_count = cur.u32()?;
            let unknown_1 = cur.u32()?;
            let mut pets_id = Vec::new();
            for _ in 0..pet_total_count {
                pets_id.push(cur.u32()?);
            }
            Ok(TileType::PetTrainer {
                label,
                pet_total_count,
                unknown_1,
                pets_id,
            })
        }
        38 => {
            // SteamEngine
            let temperature = cur.u32()?;
            Ok(TileType::SteamEngine { temperature })
        }
        39 => {
            // LockBot
            let age = cur.u32()?;
            Ok(TileType::LockBot { age })
        }
        40 => {
            // WeatherMachine
            let settings = cur.u32()?;
            Ok(TileType::WeatherMachine { settings })
        }
        41 => {
            // SpiritStorageUnit
            let ghost_jar_count = cur.u32()?;
            Ok(TileType::SpiritStorageUnit { ghost_jar_count })
        }
        42 => {
            // DataBedrock — skip 21 bytes (unk1[17] + pad1[4])
            cur.skip(21)?;
            Ok(TileType::DataBedrock)
        }
        43 => {
            // Shelf
            let top_left_item_id = cur.u32()?;
            let top_right_item_id = cur.u32()?;
            let bottom_left_item_id = cur.u32()?;
            let bottom_right_item_id = cur.u32()?;
            Ok(TileType::Shelf {
                top_left_item_id,
                top_right_item_id,
                bottom_left_item_id,
                bottom_right_item_id,
            })
        }
        44 => {
            // VipEntrance
            let unknown_1 = cur.u8()?;
            let owner_uid = cur.u32()?;
            let access_count = cur.u32()?;
            let mut access_uids = Vec::new();
            for _ in 0..access_count {
                access_uids.push(cur.u32()?);
            }
            Ok(TileType::VipEntrance {
                unknown_1,
                owner_uid,
                access_uids,
            })
        }
        45 => Ok(TileType::ChallangeTimer),
        47 => {
            // FishWallMount
            let label = cur.plain_string()?;
            let item_id = cur.u32()?;
            let lb = cur.u8()?;
            Ok(TileType::FishWallMount { label, item_id, lb })
        }
        48 => {
            // Portrait
            //      unk5(u2), face(u2), hat(u2), hair(u2), unk6(u4),
            //      then conditionally infinity_crown_data(gt_str) if hat == 12958.
            // + unk4 is u1), face/hat/hair were u32 (should be u16), unk6 was missing,
            // and the conditional infinity_crown_data was missing entirely.
            let label = cur.plain_string()?;
            let unknown_1 = cur.u32()?;
            let unknown_2 = cur.u32()?;
            let mut unknown_3 = [0u8; 5];
            for b in &mut unknown_3 {
                *b = cur.u8()?;
            }
            let unknown_4 = cur.u8()?;
            let unknown_5 = cur.u16()?;
            let face = cur.u16()?;
            let hat = cur.u16()?;
            let hair = cur.u16()?;
            let unknown_6 = cur.u32()?;
            let infinity_crown_data = if hat == 12958 {
                Some(cur.plain_string()?)
            } else {
                None
            };
            Ok(TileType::Portrait {
                label,
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
                unknown_5,
                face,
                hat,
                hair,
                unknown_6,
                infinity_crown_data,
            })
        }
        49 => {
            // WeatherMachine2, Weather Machine stuff, heatwave
            let unknown_1 = cur.u32()?;
            let gravity = cur.u32()?;
            let flags = cur.u8()?;
            Ok(TileType::WeatherMachine2 {
                unknown_1,
                gravity,
                flags,
            })
        }
        50 => {
            // FossilPrepStation
            let unknown_1 = cur.u32()?;
            Ok(TileType::FossilPrepStation { unknown_1 })
        }
        51 => Ok(TileType::DnaExtractor),
        52 => Ok(TileType::Howler),
        53 => {
            // ChemsynthTank
            let current_chem = cur.u32()?;
            let target_chem = cur.u32()?;
            Ok(TileType::ChemsynthTank {
                current_chem,
                target_chem,
            })
        }
        54 => {
            // StorageBlock
            let data_len = cur.u16()?;
            let mut items = Vec::new();
            for _ in 0..(data_len / 13) {
                cur.skip(3)?;
                let id = cur.u32()?;
                cur.skip(2)?;
                let amount = cur.u32()?;
                items.push((id, amount));
            }
            Ok(TileType::StorageBlock { items })
        }
        55 => {
            // CookingOven
            let temperature_level = cur.u32()?;
            let ingredient_count = cur.u32()?;
            let mut ingredients = Vec::new();
            for _ in 0..(ingredient_count / 2) {
                let item_id = cur.u32()?;
                let time_added = cur.u32()?;
                ingredients.push((item_id, time_added));
            }
            let unknown_1 = cur.u32()?;
            let unknown_2 = cur.u32()?;
            let unknown_3 = cur.u32()?;
            Ok(TileType::CookingOven {
                temperature_level,
                ingredients,
                unknown_1,
                unknown_2,
                unknown_3,
            })
        }
        56 => {
            // AudioRack
            let note = cur.plain_string()?;
            let volume = cur.u32()?;
            Ok(TileType::AudioRack { note, volume })
        }
        57 => {
            // GeigerCharger
            let unknown_1 = cur.u32()?;
            Ok(TileType::GeigerCharger { unknown_1 })
        }
        58 => Ok(TileType::AdventureBegins),
        59 => Ok(TileType::TombRobber),
        60 => {
            // BalloonOMatic
            let total_rarity = cur.u32()?;
            let team_type = cur.u8()?;
            Ok(TileType::BalloonOMatic {
                total_rarity,
                team_type,
            })
        }
        61 => {
            // TrainingPort
            // and unknown_3 (5 bytes) after unknown_2, both missing before.
            let fish_lb = cur.u32()?;
            let fish_status = cur.u16()?;
            let fish_id = cur.u32()?;
            let fish_total_exp = cur.u32()?;
            let mut unknown_1 = [0u8; 8];
            for b in &mut unknown_1 {
                *b = cur.u8()?;
            }
            let fish_level = cur.u32()?;
            let unknown_2 = cur.u32()?;
            let mut unknown_3 = [0u8; 5];
            for b in &mut unknown_3 {
                *b = cur.u8()?;
            }
            Ok(TileType::TrainingPort {
                fish_lb,
                fish_status,
                fish_id,
                fish_total_exp,
                unknown_1,
                fish_level,
                unknown_2,
                unknown_3,
            })
        }
        62 => {
            // ItemSucker
            let item_id_to_suck = cur.u32()?;
            let item_amount = cur.u32()?;
            let flags = cur.u16()?;
            let limit = cur.u32()?;
            Ok(TileType::ItemSucker {
                item_id_to_suck,
                item_amount,
                flags,
                limit,
            })
        }
        63 => {
            // CyBot
            // is wrong. Now commands are read first, then timer and activated after.
            let command_data_count = cur.u32()?;
            let mut command_datas = Vec::new();
            for _ in 0..command_data_count {
                let command_id = cur.u32()?;
                let is_command_used = cur.u32()?;
                cur.skip(7)?;
                command_datas.push((command_id, is_command_used));
            }
            let sync_timer = cur.u32()?;
            let activated = cur.u32()?;
            Ok(TileType::CyBot {
                command_datas,
                sync_timer,
                activated,
            })
        }
        // 64 => {
        //     // GuildBlock (kind 0x40) — reads one string at tileextra+0x28
        //     let label = cur.plain_string()?;
        //     Ok(TileType::GuildBlock { label })
        // }
        65 => {
            // GuildItem (kind 0x41) — default case in game switch, no bytes in stream
            let mut unknown_1 = [0u8; 17];
            for b in &mut unknown_1 {
                *b = cur.u8()?;
            }
            Ok(TileType::GuildItem { unknown_1 })
        }
        66 => {
            // Growscan
            let unknown_1 = cur.u8()?;
            Ok(TileType::Growscan { unknown_1 })
        }
        67 => {
            // ContainmentFieldPowerNode
            let time = cur.u32()?;
            let linked_node_count = cur.u32()?;
            let mut linked_nodes = Vec::new();
            for _ in 0..linked_node_count {
                linked_nodes.push(cur.u32()?);
            }
            Ok(TileType::ContainmentFieldPowerNode { time, linked_nodes })
        }
        68 => {
            // SpiritBoard
            // player_required(u4), unk1(gt_str), command(gt_str),
            // num_required_items(u4), required_items[].
            let player_required = cur.u32()?;
            let unk1 = cur.plain_string()?;
            let command = cur.plain_string()?;
            let num_required_items = cur.u32()?;
            let mut required_items = Vec::new();
            for _ in 0..num_required_items {
                required_items.push(cur.u32()?);
            }
            Ok(TileType::SpiritBoard {
                player_required,
                unk1,
                command,
                required_items,
            })
        }
        69 => {
            // TesseractManipulator (item 6952)
            let gems = cur.u32()?;
            let next_update_ms = cur.u32()?;
            let item_id = cur.u32()?;
            let enabled = cur.u32()?;
            Ok(TileType::TesseractManipulator {
                gems,
                next_update_ms,
                item_id,
                enabled,
            })
        }
        72 => {
            // StormyCloud
            let sting_duration = cur.u32()?;
            let is_solid = cur.u32()?;
            let non_solid_duration = cur.u32()?;
            Ok(TileType::StormyCloud {
                sting_duration,
                is_solid,
                non_solid_duration,
            })
        }
        73 => {
            // TemporaryPlatform
            let unknown_1 = cur.u32()?;
            Ok(TileType::TemporaryPlatform { unknown_1 })
        }
        74 => Ok(TileType::SafeVault),
        75 => {
            // AngelicCountingCloud
            let state = cur.u32()?;
            let unknown_1 = cur.u16()?;
            let ascii_code = if state == 2 { Some(cur.u8()?) } else { None };
            Ok(TileType::AngelicCountingCloud {
                state,
                unknown_1,
                ascii_code,
            })
        }
        77 => {
            // InfinityWeatherMachine
            let interval_minutes = cur.u32()?;
            let weather_machine_list_size = cur.u32()?;
            let mut weather_machine_list = Vec::new();
            for _ in 0..weather_machine_list_size {
                weather_machine_list.push(cur.u32()?);
            }
            Ok(TileType::InfinityWeatherMachine {
                interval_minutes,
                weather_machine_list,
            })
        }
        79 => {
            // PineappleGuzzler
            let pineapple_count = cur.u32()?;
            Ok(TileType::PineappleGuzzler { pineapple_count })
        }
        80 => {
            // KrakenGalaticBlock
            let pattern_index = cur.u8()?;
            let unknown_1 = cur.u32()?;
            let r = cur.u8()?;
            let g = cur.u8()?;
            let b = cur.u8()?;
            Ok(TileType::KrakenGalaticBlock {
                pattern_index,
                unknown_1,
                r,
                g,
                b,
            })
        }
        81 => {
            // FriendsEntrance
            let owner_user_id = cur.u32()?;
            cur.skip(2)?;
            let num_allowed = cur.u16()? as u32;
            let mut allowed_friends_userid = Vec::new();
            for _ in 0..num_allowed {
                allowed_friends_userid.push(cur.u32()?);
            }
            Ok(TileType::FriendsEntrance {
                owner_user_id,
                allowed_friends_userid,
            })
        }
        _ => {
            // Unknown / unhandled kind — log and return marker (no bytes consumed beyond kind)
            eprintln!(
                "[world] WARNING: unknown TileExtraData kind {kind:#x} at fg_item={fg_item_id}"
            );
            Ok(TileType::Unknown { kind })
        }
    }
}

// ── WorldNpc ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NpcAction {
    FullState       = 0,
    Delete          = 1,
    Add             = 2,
    MoveTo          = 3,
    Sucked          = 4,
    Burp            = 5,
    Teleport        = 6,
    Die             = 7,
    Punch           = 8,
    Ouch            = 9,
    Attack          = 10,
    PrepareToAttack = 11,
}

impl NpcAction {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0  => Some(Self::FullState),
            1  => Some(Self::Delete),
            2  => Some(Self::Add),
            3  => Some(Self::MoveTo),
            4  => Some(Self::Sucked),
            5  => Some(Self::Burp),
            6  => Some(Self::Teleport),
            7  => Some(Self::Die),
            8  => Some(Self::Punch),
            9  => Some(Self::Ouch),
            10 => Some(Self::Attack),
            11 => Some(Self::PrepareToAttack),
            _  => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NpcType {
    None                                    = 0,
    Ghost                                   = 1,
    GhostJar                                = 2,
    BeeSwarm                                = 3,
    HarvestGhost                            = 4,
    GrowGa                                  = 5,
    GhostShark                              = 6,
    XmasGhost                               = 7,
    Blast                                   = 8,
    Pinata                                  = 9,
    GhostCaptureMachine                     = 10,
    BossGhost                               = 11,
    MindControlGhost                        = 12,
    GhostBeGone                             = 13,
    HuntedTurkey                            = 14,
    Trickster                               = 15,
    ThanksgivingTurkeyBoss                  = 16,
    ThanksgivingTurkeyBossFeatherProjectile = 17,
    AttackerMinionTurkey                    = 18,
    BeachEnemy                              = 19,
    XmlConfigured                           = 20,
    XmlRendered                             = 21,
}

impl NpcType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1  => Self::Ghost,
            2  => Self::GhostJar,
            3  => Self::BeeSwarm,
            4  => Self::HarvestGhost,
            5  => Self::GrowGa,
            6  => Self::GhostShark,
            7  => Self::XmasGhost,
            8  => Self::Blast,
            9  => Self::Pinata,
            10 => Self::GhostCaptureMachine,
            11 => Self::BossGhost,
            12 => Self::MindControlGhost,
            13 => Self::GhostBeGone,
            14 => Self::HuntedTurkey,
            15 => Self::Trickster,
            16 => Self::ThanksgivingTurkeyBoss,
            17 => Self::ThanksgivingTurkeyBossFeatherProjectile,
            18 => Self::AttackerMinionTurkey,
            19 => Self::BeachEnemy,
            20 => Self::XmlConfigured,
            21 => Self::XmlRendered,
            _  => Self::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorldNpc {
    pub npc_type: NpcType,
    pub id:       u8,
    pub x:        f32,
    pub y:        f32,
    pub dest_x:   f32,
    pub dest_y:   f32,
    pub unk1:     f32,
    pub unk2:     f32,
    pub var:      f32,
}

// ── WorldObject ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WorldObject {
    pub item_id: u16,
    pub x: f32,
    pub y: f32,
    pub count: u8,
    pub flags: u8,
    pub uid: u32,
}

fn parse_world_objects(cur: &mut Cursor) -> Result<(Vec<WorldObject>, u32)> {
    let count = cur.u32()?;
    let last_dropped_uid = cur.u32()?;

    if count >= MAX_WORLD_OBJECTS {
        bail!("world object count {count} >= limit {MAX_WORLD_OBJECTS}");
    }

    let mut objects = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let item_id = cur.u16()?;
        let x = cur.f32()?;
        let y = cur.f32()?;
        let count_ = cur.u8()?;
        let flags = cur.u8()?;
        let uid = cur.u32()?;

        let item_id = if matches!(item_id, 5996 | 1626) {
            0
        } else {
            item_id
        };
        objects.push(WorldObject {
            item_id,
            x,
            y,
            count: count_,
            flags,
            uid,
        });
    }

    Ok((objects, last_dropped_uid))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_world_dat() {
        let data = match std::fs::read("world.dat") {
            Ok(d) => d,
            Err(_) => {
                println!("world.dat not found — skipping");
                return;
            }
        };

        let world = World::parse(&data).expect("World::parse failed");

        println!("version      : {:#x}", world.version);
        println!("world_flags  : {:#x}", world.flags);
        println!("world_name   : {:?}", world.tile_map.world_name);
        println!("width        : {}", world.tile_map.width);
        println!("height       : {}", world.tile_map.height);
        println!("tiles        : {}", world.tile_map.tiles.len());
        println!("objects      : {}", world.objects.len());
        println!("base_weather : {}", world.base_weather);
        println!("cur_weather  : {}", world.current_weather);

        assert_eq!(world.version, 0x19);
        assert_eq!(
            world.tile_map.tiles.len() as u32,
            world.tile_map.width * world.tile_map.height
        );

        assert!(!world.tile_map.world_name.is_empty(), "world name should be present");
        assert!(
            world.tile_map.tiles.iter().any(|tile| tile.fg_item_id != 0),
            "sample world should contain at least one non-empty foreground tile"
        );
        assert!(
            world.objects.len() <= MAX_WORLD_OBJECTS as usize,
            "parsed object count should stay within parser limits"
        );
    }
}
