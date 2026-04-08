use anyhow::Result;
use crate::cursor::Cursor;

#[derive(Debug, Clone, Copy, PartialEq)]
enum VariantType {
    Unknown,
    Float,
    String,
    Vec2,
    Vec3,
    Unsigned,
    Signed,
}

impl From<u8> for VariantType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Float,
            2 => Self::String,
            3 => Self::Vec2,
            4 => Self::Vec3,
            5 => Self::Unsigned,
            9 => Self::Signed,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Variant {
    Float(f32),
    String(String),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Unsigned(u32),
    Signed(i32),
    Unknown,
}

impl Variant {
    pub fn as_string(&self) -> String {
        match self {
            Self::Float(v)        => v.to_string(),
            Self::String(v)       => v.clone(),
            Self::Vec2(x, y)      => format!("{x}, {y}"),
            Self::Vec3(x, y, z)   => format!("{x}, {y}, {z}"),
            Self::Unsigned(v)     => v.to_string(),
            Self::Signed(v)       => v.to_string(),
            Self::Unknown         => String::new(),
        }
    }

    pub fn as_int32(&self) -> i32 {
        match self {
            Self::Signed(v) => *v,
            _ => 0,
        }
    }

    pub fn as_uint32(&self) -> u32 {
        match self {
            Self::Unsigned(v) => *v,
            _ => 0,
        }
    }

    pub fn as_vec2(&self) -> (f32, f32) {
        match self {
            Self::Vec2(x, y) => (*x, *y),
            _ => (0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariantList {
    variants: Vec<Variant>,
}

impl VariantList {
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data, "variant");
        let count      = cursor.u8()? as usize;
        let mut variants = Vec::with_capacity(count);

        for _ in 0..count {
            let _index   = cursor.u8()?;
            let var_type = VariantType::from(cursor.u8()?);

            let variant = match var_type {
                VariantType::Float => {
                    Variant::Float(cursor.f32()?)
                }
                VariantType::String => {
                    let len = cursor.u32()? as usize;
                    Variant::String(String::from_utf8_lossy(&cursor.bytes(len)?).into_owned())
                }
                VariantType::Vec2 => {
                    let x = cursor.f32()?;
                    let y = cursor.f32()?;
                    Variant::Vec2(x, y)
                }
                VariantType::Vec3 => {
                    let x = cursor.f32()?;
                    let y = cursor.f32()?;
                    let z = cursor.f32()?;
                    Variant::Vec3(x, y, z)
                }
                VariantType::Unsigned => {
                    Variant::Unsigned(cursor.u32()?)
                }
                VariantType::Signed => {
                    Variant::Signed(cursor.i32()?)
                }
                VariantType::Unknown => Variant::Unknown,
            };

            variants.push(variant);
        }

        Ok(Self { variants })
    }

    pub fn get(&self, index: usize) -> Option<&Variant> {
        self.variants.get(index)
    }
}
