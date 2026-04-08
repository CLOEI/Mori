use std::collections::BTreeSet;
use std::fmt;

use anyhow::{bail, Result};
use crate::cursor::Cursor;

// ── XOR encode/decode (key "90210") ───────────────────────────────────────────
// Self-inverse: encode == decode. Used for `meta`, `tankid_password`, `parentalpass`.

fn xor_90210(data: &[u8]) -> Vec<u8> {
    const KEY: &[u8] = b"90210";
    data.iter().enumerate().map(|(i, &b)| b ^ KEY[i % KEY.len()]).collect()
}

// ── Variant value ─────────────────────────────────────────────────────────────

/// String variant holds raw bytes — save.dat strings are not guaranteed to be UTF-8
/// (meta and seed_diary_data contain binary/XOR-encoded content).
#[derive(Clone, PartialEq)]
pub enum VariantValue {
    Float(f32),
    String(Vec<u8>),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Uint(u32),
    Rect(f32, f32, f32, f32),
    Int(i32),
}

impl fmt::Debug for VariantValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariantValue::Float(v)          => write!(f, "Float({v})"),
            VariantValue::String(b)         => write!(f, "String({:?})", String::from_utf8_lossy(b)),
            VariantValue::Vec2(x, y)        => write!(f, "Vec2({x}, {y})"),
            VariantValue::Vec3(x, y, z)     => write!(f, "Vec3({x}, {y}, {z})"),
            VariantValue::Uint(v)           => write!(f, "Uint({v})"),
            VariantValue::Rect(x, y, w, h)  => write!(f, "Rect({x}, {y}, {w}, {h})"),
            VariantValue::Int(v)            => write!(f, "Int({v})"),
        }
    }
}

// ── Entry ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Entry {
    pub key:   String,
    pub value: VariantValue,
}

// ── Seed diary ────────────────────────────────────────────────────────────────

pub const SEED_DIARY_MAX_ID: u16 = 16010; // 0x3E8A

/// Decoded view of the `seed_diary_data` save.dat field.
///
/// On-disk format: packed 16-bit LE entries, one per "have" item.
/// ```text
/// bits [14:0] = item_id   (0 – 16010)
/// bit  [15]   = grown_flag
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SeedDiary {
    pub have:  BTreeSet<u16>,
    pub grown: BTreeSet<u16>,
}

impl SeedDiary {
    pub fn parse(data: &[u8]) -> Self {
        let mut diary = SeedDiary::default();
        let mut i = 0;
        while i + 1 < data.len() {
            let lo = data[i];
            let hi = data[i + 1];
            let item_id = lo as u16 | ((hi & 0x7F) as u16) << 8;
            if item_id <= SEED_DIARY_MAX_ID {
                diary.have.insert(item_id);
                if hi & 0x80 != 0 {
                    diary.grown.insert(item_id);
                }
            }
            i += 2;
        }
        diary
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for id in 0..=SEED_DIARY_MAX_ID {
            if self.have.contains(&id) {
                let lo = (id & 0xFF) as u8;
                let hi = ((id >> 8) & 0x7F) as u8
                    | if self.grown.contains(&id) { 0x80 } else { 0 };
                buf.push(lo);
                buf.push(hi);
            }
        }
        buf
    }
}

// ── Top-level container ───────────────────────────────────────────────────────

pub struct SaveDat {
    pub entries: Vec<Entry>,
}

impl SaveDat {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn set(&mut self, key: impl Into<String>, value: VariantValue) {
        let key = key.into();
        if let Some(e) = self.entries.iter_mut().find(|e| e.key == key) {
            e.value = value;
        } else {
            self.entries.push(Entry { key, value });
        }
    }

    pub fn get(&self, key: &str) -> Option<&VariantValue> {
        self.entries.iter().find(|e| e.key == key).map(|e| &e.value)
    }

    /// Returns the XOR-decoded bytes of the `meta` field, or `None` if absent.
    pub fn get_meta(&self) -> Option<Vec<u8>> {
        match self.get("meta") {
            Some(VariantValue::String(b)) => Some(xor_90210(b)),
            _ => None,
        }
    }

    /// XOR-encodes `plain` with key `90210` and stores it under `meta`.
    pub fn set_meta(&mut self, plain: &[u8]) {
        self.set("meta", VariantValue::String(xor_90210(plain)));
    }

    /// Parses the `seed_diary_data` field into a [`SeedDiary`], or `None` if absent.
    pub fn get_seed_diary(&self) -> Option<SeedDiary> {
        match self.get("seed_diary_data") {
            Some(VariantValue::String(b)) => Some(SeedDiary::parse(b)),
            _ => None,
        }
    }

    /// Serializes `diary` and stores it under `seed_diary_data`.
    pub fn set_seed_diary(&mut self, diary: &SeedDiary) {
        self.set("seed_diary_data", VariantValue::String(diary.serialize()));
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1u32.to_le_bytes());
        for entry in &self.entries {
            let type_id: u32 = match &entry.value {
                VariantValue::Float(_)          => 1,
                VariantValue::String(_)         => 2,
                VariantValue::Vec2(_, _)        => 3,
                VariantValue::Vec3(_, _, _)     => 4,
                VariantValue::Uint(_)           => 5,
                VariantValue::Rect(_, _, _, _)  => 8,
                VariantValue::Int(_)            => 9,
            };
            buf.extend_from_slice(&type_id.to_le_bytes());
            buf.extend_from_slice(&(entry.key.len() as u32).to_le_bytes());
            buf.extend_from_slice(entry.key.as_bytes());
            match &entry.value {
                VariantValue::Float(v)          => buf.extend_from_slice(&v.to_le_bytes()),
                VariantValue::String(b)         => {
                    buf.extend_from_slice(&(b.len() as u32).to_le_bytes());
                    buf.extend_from_slice(b);
                }
                VariantValue::Vec2(x, y)        => {
                    buf.extend_from_slice(&x.to_le_bytes());
                    buf.extend_from_slice(&y.to_le_bytes());
                }
                VariantValue::Vec3(x, y, z)     => {
                    buf.extend_from_slice(&x.to_le_bytes());
                    buf.extend_from_slice(&y.to_le_bytes());
                    buf.extend_from_slice(&z.to_le_bytes());
                }
                VariantValue::Uint(v)           => buf.extend_from_slice(&v.to_le_bytes()),
                VariantValue::Rect(x, y, w, h)  => {
                    buf.extend_from_slice(&x.to_le_bytes());
                    buf.extend_from_slice(&y.to_le_bytes());
                    buf.extend_from_slice(&w.to_le_bytes());
                    buf.extend_from_slice(&h.to_le_bytes());
                }
                VariantValue::Int(v)            => buf.extend_from_slice(&v.to_le_bytes()),
            }
        }
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf
    }

    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cur = Cursor::new(data, "save.dat");

        let magic = cur.u32()?;
        if magic != 1 {
            bail!("unexpected magic: {magic}");
        }

        let mut entries = Vec::new();
        while cur.remaining() > 0 {
            let type_id = cur.u32()?;
            if type_id == 0 { break; }
            let key_len = cur.u32()? as usize;
            let key     = cur.string_raw(key_len)?;
            let value   = match type_id {
                1 => VariantValue::Float(cur.f32()?),
                2 => {
                    let len = cur.u32()? as usize;
                    VariantValue::String(cur.bytes(len)?)
                }
                3 => VariantValue::Vec2(cur.f32()?, cur.f32()?),
                4 => VariantValue::Vec3(cur.f32()?, cur.f32()?, cur.f32()?),
                5 => VariantValue::Uint(cur.u32()?),
                8 => VariantValue::Rect(cur.f32()?, cur.f32()?, cur.f32()?, cur.f32()?),
                9 => VariantValue::Int(cur.i32()?),
                _ => bail!("unknown variant type {type_id} for key {key:?}"),
            };
            entries.push(Entry { key, value });
        }

        Ok(Self { entries })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn read_save_dat() -> Option<Vec<u8>> {
        match std::fs::read("save.dat") {
            Ok(data) => Some(data),
            Err(_) => {
                println!("save.dat not found - skipping");
                None
            }
        }
    }

    #[test]
    fn parse_save_dat() {
        let Some(data) = read_save_dat() else {
            return;
        };
        let save = SaveDat::parse(&data).expect("parse failed");

        println!("entries: {}", save.entries.len());
        for entry in &save.entries {
            println!("  {:?} = {:?}", entry.key, entry.value);
        }

        assert!(!save.entries.is_empty());
    }

    #[test]
    fn roundtrip_save_dat() {
        let Some(data) = read_save_dat() else {
            return;
        };
        let original = SaveDat::parse(&data).expect("parse failed");
        let reserialized = original.serialize();
        let reparsed = SaveDat::parse(&reserialized).expect("re-parse failed");

        assert_eq!(original.entries.len(), reparsed.entries.len(), "entry count mismatch");
        for (a, b) in original.entries.iter().zip(reparsed.entries.iter()) {
            assert_eq!(a.key, b.key, "key mismatch");
            assert_eq!(a.value, b.value, "value mismatch for key {:?}", a.key);
        }
    }

    #[test]
    fn serialize_set() {
        let mut save = SaveDat::new();
        save.set("Token",      VariantValue::String(b"abc".to_vec()));
        save.set("Client",     VariantValue::Uint(0x20));
        save.set("player_age", VariantValue::Int(18));

        let parsed = SaveDat::parse(&save.serialize()).expect("parse failed");

        assert_eq!(parsed.get("Token"),      Some(&VariantValue::String(b"abc".to_vec())));
        assert_eq!(parsed.get("Client"),     Some(&VariantValue::Uint(0x20)));
        assert_eq!(parsed.get("player_age"), Some(&VariantValue::Int(18)));
    }

    #[test]
    fn meta_xor_roundtrip() {
        let plain = b"some-server-meta-token-123";

        let mut save = SaveDat::new();
        save.set_meta(plain);

        // raw stored bytes must differ from plaintext
        match save.get("meta") {
            Some(VariantValue::String(b)) => assert_ne!(b.as_slice(), plain),
            _ => panic!("meta not found"),
        }

        // decode must recover original
        assert_eq!(save.get_meta().as_deref(), Some(plain.as_slice()));
    }

    #[test]
    fn meta_from_save_dat() {
        let Some(data) = read_save_dat() else {
            return;
        };
        let save = SaveDat::parse(&data).expect("parse failed");

        if let Some(decoded) = save.get_meta() {
            println!("meta (decoded, {} bytes): {:?}", decoded.len(), String::from_utf8_lossy(&decoded));
        } else {
            println!("meta key absent");
        }
    }

    #[test]
    fn seed_diary_roundtrip() {
        let mut diary = SeedDiary::default();
        diary.have.insert(0);
        diary.have.insert(100);
        diary.have.insert(999);
        diary.have.insert(16010);
        diary.grown.insert(100);
        diary.grown.insert(16010);

        let bytes     = diary.serialize();
        let recovered = SeedDiary::parse(&bytes);

        assert_eq!(diary, recovered);
    }

    #[test]
    fn seed_diary_from_save_dat() {
        let Some(data) = read_save_dat() else {
            return;
        };
        let save = SaveDat::parse(&data).expect("parse failed");

        if let Some(diary) = save.get_seed_diary() {
            println!("seed diary: {} have, {} grown", diary.have.len(), diary.grown.len());
            for id in &diary.have {
                let g = if diary.grown.contains(id) { " (grown)" } else { "" };
                println!("  item {id}{g}");
            }
        } else {
            println!("seed_diary_data absent");
        }
    }
}
