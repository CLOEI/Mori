use anyhow::Result;
use crate::cursor::Cursor;

const XOR_KEY: &[u8; 16] = b"PBG892FXX982ABC*";

// ── Item record ───────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ItemInfo {
    pub id:                  u32,
    pub flags:               u16,
    pub action_type:         u8,
    pub material:            u8,
    pub name:                String,
    pub texture_file_name:   String,
    pub texture_hash:        u32,
    pub visual_effect:       u8,
    pub cooking_ingredient:  u32,
    pub texture_x:           u8,
    pub texture_y:           u8,
    pub render_type:         u8,
    pub is_stripey_wallpaper: u8,
    pub collision_type:      u8,
    pub block_health:        u8,
    pub drop_chance:         u32,
    pub clothing_type:       u8,
    pub rarity:              u16,
    pub max_item:            u8,
    pub file_name:           String,
    pub file_hash:           u32,
    pub audio_volume:        u32,
    pub pet_name:            String,
    pub pet_prefix:          String,
    pub pet_suffix:          String,
    pub pet_ability:         String,
    pub seed_base_sprite:    u8,
    pub seed_overlay_sprite: u8,
    pub tree_base_sprite:    u8,
    pub tree_overlay_sprite: u8,
    pub base_color:          u32,
    pub overlay_color:       u32,
    pub ingredient:          u32,
    pub grow_time:           u32,
    pub is_rayman:           u16,
    pub extra_options:       String,
    pub texture_path_2:      String,
    pub extra_option2:       String,
    // v11+
    pub punch_option:        String,
    // v22+
    pub description:         String,
    pub hit_sound_fx: String,
    pub hit_sound_fx_hash: u32,
}

// ── Color helpers ─────────────────────────────────────────────────────────────

/// Items.dat packs colors as BGRA (MSB→LSB). Returns `(b, g, r, a)`.
pub fn extract_bgra(color: u32) -> (u8, u8, u8, u8) {
    let b = (color >> 24) as u8;
    let g = ((color >> 16) & 0xFF) as u8;
    let r = ((color >> 8) & 0xFF) as u8;
    let a = (color & 0xFF) as u8;
    (b, g, r, a)
}

/// Convert a BGRA-packed color to a `0xRRGGBB` value suitable for the minimap.
pub fn bgra_to_rgb(color: u32) -> u32 {
    let (b, g, r, _) = extract_bgra(color);
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

// ── Top-level container ───────────────────────────────────────────────────────

pub struct ItemsDat {
    pub version: u16,
    pub items:   Vec<ItemInfo>,
}

impl ItemsDat {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cur = Cursor::new(data, "items.dat");
        let version    = cur.u16()?;
        let item_count = cur.u32()?;
        let mut items = Vec::with_capacity(item_count as usize);
        for _ in 0..item_count {
            items.push(parse_item(&mut cur, version)?);
        }
        Ok(Self { version, items })
    }

    pub fn find_by_id(&self, id: u32) -> Option<&ItemInfo> {
        self.items
            .get(id as usize)
            .filter(|i| i.id == id)
            .or_else(|| self.items.iter().find(|i| i.id == id))
    }

    pub fn find_by_name(&self, name: &str) -> Option<&ItemInfo> {
        let lower = name.to_lowercase();
        self.items.iter().find(|i| i.name.to_lowercase() == lower)
    }

    /// Load from `items.dat` on disk; returns an empty database on failure.
    pub fn load() -> Self {
        match std::fs::read("items.dat").and_then(|d| Self::parse(&d).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))) {
            Ok(db) => {
                println!("[Items] Loaded {} items", db.items.len());
                db
            }
            Err(e) => {
                println!("[Items] Failed to load items.dat: {e} — pathfinding will be inaccurate");
                Self { version: 0, items: Vec::new() }
            }
        }
    }
}

// ── Per-item parser ───────────────────────────────────────────────────────────

fn parse_item(cur: &mut Cursor, version: u16) -> Result<ItemInfo> {
    let mut it = ItemInfo::default();

    it.id          = cur.u32()?;
    it.flags       = cur.u16()?;
    it.action_type = cur.u8()?;
    it.material    = cur.u8()?;

    it.name              = cur.xor_string(XOR_KEY, (it.id % 16) as usize)?;
    it.texture_file_name = cur.plain_string()?;
    it.texture_hash      = cur.u32()?;
    it.visual_effect     = cur.u8()?;
    it.cooking_ingredient = cur.u32()?;
    it.texture_x         = cur.u8()?;
    it.texture_y         = cur.u8()?;
    it.render_type       = cur.u8()?;
    it.is_stripey_wallpaper = cur.u8()?;
    it.collision_type    = cur.u8()?;
    it.block_health      = cur.u8()?;
    it.drop_chance       = cur.u32()?;
    it.clothing_type     = cur.u8()?;
    it.rarity            = cur.u16()?;
    it.max_item          = cur.u8()?;

    it.file_name     = cur.plain_string()?;
    it.file_hash     = cur.u32()?;
    it.audio_volume  = cur.u32()?;

    it.pet_name    = cur.plain_string()?;
    it.pet_prefix  = cur.plain_string()?;
    it.pet_suffix  = cur.plain_string()?;
    it.pet_ability = cur.plain_string()?;

    it.seed_base_sprite    = cur.u8()?;
    it.seed_overlay_sprite = cur.u8()?;
    it.tree_base_sprite    = cur.u8()?;
    it.tree_overlay_sprite = cur.u8()?;
    it.base_color          = cur.u32()?;
    it.overlay_color       = cur.u32()?;
    it.ingredient          = cur.u32()?;
    it.grow_time           = cur.u32()?;

    cur.skip(2)?; // unused u16
    it.is_rayman = cur.u16()?;

    it.extra_options  = cur.plain_string()?;
    it.texture_path_2 = cur.plain_string()?;
    it.extra_option2  = cur.plain_string()?;

    cur.skip(80)?;

    if version >= 11 {
        it.punch_option = cur.plain_string()?;
    }
    if version >= 12 {
        cur.skip(13)?;
    }
    if version >= 13 {
        cur.skip(4)?;
    }
    if version >= 14 {
        cur.skip(4)?;
    }
    if version >= 15 {
        cur.skip(25)?;
        cur.plain_string()?;
    }
    if version >= 16 {
        cur.plain_string()?;
    }
    if version >= 17 {
        cur.skip(4)?;
    }
    if version >= 18 {
        cur.skip(4)?;
    }
    if version >= 19 {
        cur.skip(9)?;
    }
    if version >= 21 {
        cur.skip(2)?;
    }
    if version >= 22 {
        it.description = cur.plain_string()?;
    }
    if version >= 23 {
        cur.skip(4)?;
    }
    if version >= 24 {
        cur.skip(1)?;
    }
    if version >= 25 {
        it.hit_sound_fx = cur.plain_string()?;
        it.hit_sound_fx_hash = cur.u32()?;
    }
    if version >= 26 {
        cur.skip(1)?;
    }

    Ok(it)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_items_dat() {
        let data = std::fs::read("items.dat").expect("items.dat not found in project root");
        let db = ItemsDat::parse(&data).expect("parse failed");

        println!("version    : {}", db.version);
        println!("item count : {}", db.items.len());

        let i0 = &db.items[0];
        println!("item[0]    : id={} name={:?} material={} action={} flags={}",
            i0.id, i0.name, i0.material, i0.action_type, i0.flags);

        for id in [1, 2, 32, 100, 242] {
            if let Some(it) = db.find_by_id(id) {
                println!("item[{:3}]   : name={:?} texture={:?} rarity={} grow_time={}",
                    it.id, it.name, it.texture_file_name, it.rarity, it.grow_time);
            }
        }

        assert!(!db.items.is_empty());
        assert!(db.items.last().unwrap().name != "");
    }
}
