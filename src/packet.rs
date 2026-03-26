use std::fmt;

// ── Outer message types ──────────────────────────────────────────────────────

pub const MSG_SERVER_HELLO: u32 = 1;
pub const MSG_TEXT: u32 = 2;
pub const MSG_GAME_MESSAGE: u32 = 3;
pub const MSG_GAME_PACKET: u32 = 4;
pub const MSG_TRACK: u32 = 6;
pub const MSG_CLIENT_LOG_REQUEST: u32 = 7;

// ── NET_GAME_PACKET subtypes ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum GamePacketType {
    #[default]
    State                          = 0x00,
    CallFunction                   = 0x01,
    UpdateStatus                   = 0x02,
    TileChangeRequest              = 0x03,
    SendMapData                    = 0x04,
    SendTileUpdateData             = 0x05,
    SendTileUpdateDataMultiple     = 0x06,
    TileActivateRequest            = 0x07,
    TileApplyDamage                = 0x08,
    SendInventoryState             = 0x09,
    ItemActivateObjectRequest      = 0x0A,
    ItemActivateObjectRequestAlt   = 0x0B,
    SendTileTreeState              = 0x0C,
    ModifyItemInventory            = 0x0D,
    ItemChangeObject               = 0x0E,
    SendLock                       = 0x0F,
    SendItemDatabaseData           = 0x10,
    SendParticleEffect             = 0x11,
    SetIconState                   = 0x12,
    ItemEffect                     = 0x13,
    SetCharacterState              = 0x14,
    PingReply                      = 0x15,
    PingRequest                    = 0x16,
    GotPunched                     = 0x17,
    AppCheckResponse               = 0x18,
    AppIntegrityFail               = 0x19,
    Disconnect                     = 0x1A,
    BattleJoin                     = 0x1B,
    BattleEvent                    = 0x1C,
    UseDoor                        = 0x1D,
    SendParental                   = 0x1E,
    GoneFishin                     = 0x1F,
    Steam                          = 0x20,
    PetBattle                      = 0x21,
    Npc                            = 0x22,
    Special                        = 0x23,
    SendParticleEffectV2           = 0x24,
    ActiveArrowToItem              = 0x25,
    SelectTileIndex                = 0x26,
    SendPlayerTributeData          = 0x27,
    FtueSetItemToQuickInventory    = 0x28,
    PveNpc                         = 0x29,
    PvpCardBattle                  = 0x2A,
    PveApplyPlayerDamage           = 0x2B,
    PveNpcPositionUpdate           = 0x2C,
    SetExtraMods                   = 0x2D,
    OnStepOnTileMod                = 0x2E,
    Unknown(u8),
}

impl GamePacketType {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::State                          => 0x00,
            Self::CallFunction                   => 0x01,
            Self::UpdateStatus                   => 0x02,
            Self::TileChangeRequest              => 0x03,
            Self::SendMapData                    => 0x04,
            Self::SendTileUpdateData             => 0x05,
            Self::SendTileUpdateDataMultiple     => 0x06,
            Self::TileActivateRequest            => 0x07,
            Self::TileApplyDamage                => 0x08,
            Self::SendInventoryState             => 0x09,
            Self::ItemActivateObjectRequest      => 0x0A,
            Self::ItemActivateObjectRequestAlt   => 0x0B,
            Self::SendTileTreeState              => 0x0C,
            Self::ModifyItemInventory            => 0x0D,
            Self::ItemChangeObject               => 0x0E,
            Self::SendLock                       => 0x0F,
            Self::SendItemDatabaseData           => 0x10,
            Self::SendParticleEffect             => 0x11,
            Self::SetIconState                   => 0x12,
            Self::ItemEffect                     => 0x13,
            Self::SetCharacterState              => 0x14,
            Self::PingReply                      => 0x15,
            Self::PingRequest                    => 0x16,
            Self::GotPunched                     => 0x17,
            Self::AppCheckResponse               => 0x18,
            Self::AppIntegrityFail               => 0x19,
            Self::Disconnect                     => 0x1A,
            Self::BattleJoin                     => 0x1B,
            Self::BattleEvent                    => 0x1C,
            Self::UseDoor                        => 0x1D,
            Self::SendParental                   => 0x1E,
            Self::GoneFishin                     => 0x1F,
            Self::Steam                          => 0x20,
            Self::PetBattle                      => 0x21,
            Self::Npc                            => 0x22,
            Self::Special                        => 0x23,
            Self::SendParticleEffectV2           => 0x24,
            Self::ActiveArrowToItem              => 0x25,
            Self::SelectTileIndex                => 0x26,
            Self::SendPlayerTributeData          => 0x27,
            Self::FtueSetItemToQuickInventory    => 0x28,
            Self::PveNpc                         => 0x29,
            Self::PvpCardBattle                  => 0x2A,
            Self::PveApplyPlayerDamage           => 0x2B,
            Self::PveNpcPositionUpdate           => 0x2C,
            Self::SetExtraMods                   => 0x2D,
            Self::OnStepOnTileMod                => 0x2E,
            Self::Unknown(v)                     => v,
        }
    }
}

impl From<u8> for GamePacketType {
    fn from(v: u8) -> Self {
        match v {
            0x00 => Self::State,
            0x01 => Self::CallFunction,
            0x02 => Self::UpdateStatus,
            0x03 => Self::TileChangeRequest,
            0x04 => Self::SendMapData,
            0x05 => Self::SendTileUpdateData,
            0x06 => Self::SendTileUpdateDataMultiple,
            0x07 => Self::TileActivateRequest,
            0x08 => Self::TileApplyDamage,
            0x09 => Self::SendInventoryState,
            0x0A => Self::ItemActivateObjectRequest,
            0x0B => Self::ItemActivateObjectRequestAlt,
            0x0C => Self::SendTileTreeState,
            0x0D => Self::ModifyItemInventory,
            0x0E => Self::ItemChangeObject,
            0x0F => Self::SendLock,
            0x10 => Self::SendItemDatabaseData,
            0x11 => Self::SendParticleEffect,
            0x12 => Self::SetIconState,
            0x13 => Self::ItemEffect,
            0x14 => Self::SetCharacterState,
            0x15 => Self::PingReply,
            0x16 => Self::PingRequest,
            0x17 => Self::GotPunched,
            0x18 => Self::AppCheckResponse,
            0x19 => Self::AppIntegrityFail,
            0x1A => Self::Disconnect,
            0x1B => Self::BattleJoin,
            0x1C => Self::BattleEvent,
            0x1D => Self::UseDoor,
            0x1E => Self::SendParental,
            0x1F => Self::GoneFishin,
            0x20 => Self::Steam,
            0x21 => Self::PetBattle,
            0x22 => Self::Npc,
            0x23 => Self::Special,
            0x24 => Self::SendParticleEffectV2,
            0x25 => Self::ActiveArrowToItem,
            0x26 => Self::SelectTileIndex,
            0x27 => Self::SendPlayerTributeData,
            0x28 => Self::FtueSetItemToQuickInventory,
            0x29 => Self::PveNpc,
            0x2A => Self::PvpCardBattle,
            0x2B => Self::PveApplyPlayerDamage,
            0x2C => Self::PveNpcPositionUpdate,
            0x2D => Self::SetExtraMods,
            0x2E => Self::OnStepOnTileMod,
            other => Self::Unknown(other),
        }
    }
}

// ── GameUpdatePacket flags ───────────────────────────────────────────────────

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PacketFlags: u32 {
        const WALK                   = 0x0000_0001;
        const UNK_2                  = 0x0000_0002;
        const SPAWN_RELATED          = 0x0000_0004;
        const EXTENDED               = 0x0000_0008;
        const FACING_LEFT            = 0x0000_0010;
        const STANDING               = 0x0000_0020;
        const FIRE_DAMAGE            = 0x0000_0040;
        const JUMP                   = 0x0000_0080;
        const GOT_KILLED             = 0x0000_0100;
        const PUNCH                  = 0x0000_0200;
        const PLACE                  = 0x0000_0400;
        const TILE_CHANGE            = 0x0000_0800;
        const GOT_PUNCHED            = 0x0000_1000;
        const RESPAWN                = 0x0000_2000;
        const OBJECT_COLLECT         = 0x0000_4000;
        const TRAMPOLINE             = 0x0000_8000;
        const DAMAGE                 = 0x0001_0000;
        const SLIDE                  = 0x0002_0000;
        const PARASOL                = 0x0004_0000;
        const UNK_GRAVITY_RELATED    = 0x0008_0000;
        const SWIM                   = 0x0010_0000;
        const WALL_HANG              = 0x0020_0000;
        const POWER_UP_PUNCH_START   = 0x0040_0000;
        const POWER_UP_PUNCH_END     = 0x0080_0000;
        const UNK_TILE_CHANGE        = 0x0100_0000;
        const HAY_CART_RELATED       = 0x0200_0000;
        const ACID_RELATED_DAMAGE    = 0x0400_0000;
        const UNK_3                  = 0x0800_0000;
        const ACID_DAMAGE            = 0x1000_0000;
    }
}

// ── GameUpdatePacket (56-byte wire layout) ───────────────────────────────────

pub const GAME_PACKET_SIZE: usize = 56;

#[derive(Debug, Clone, Default)]
pub struct GameUpdatePacket {
    pub packet_type:       GamePacketType,
    pub object_type:       u8,   // byte [1]
    pub jump_count:        u8,   // byte [2]
    pub animation_type:    u8,   // byte [3]
    pub net_id:            u32,
    pub target_net_id:     i32,
    pub flags:             PacketFlags,
    pub float_variable:    f32,
    pub value:             u32,
    pub vector_x:          f32,
    pub vector_y:          f32,
    pub vector_x2:         f32,
    pub vector_y2:         f32,
    pub particle_rotation: f32,
    pub int_x:             i32,
    pub int_y:             i32,
    /// Present when `flags.contains(PacketFlags::EXTENDED)`
    pub extra_data:        Vec<u8>,
}

impl GameUpdatePacket {
    /// Parse from raw ENet payload (everything after the 4-byte outer type).
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < GAME_PACKET_SIZE {
            return None;
        }

        let read_u32 = |off: usize| u32::from_le_bytes(data[off..off + 4].try_into().unwrap());
        let read_i32 = |off: usize| i32::from_le_bytes(data[off..off + 4].try_into().unwrap());
        let read_f32 = |off: usize| f32::from_le_bytes(data[off..off + 4].try_into().unwrap());

        let packet_type       = GamePacketType::from(data[0]);
        let object_type       = data[1];
        let jump_count        = data[2];
        let animation_type    = data[3];
        let net_id            = read_u32(4);
        let target_net_id     = read_i32(8);
        let flags             = PacketFlags::from_bits_retain(read_u32(12));
        let float_variable    = read_f32(16);
        let value             = read_u32(20);
        let vector_x          = read_f32(24);
        let vector_y          = read_f32(28);
        let vector_x2         = read_f32(32);
        let vector_y2         = read_f32(36);
        let particle_rotation = read_f32(40);
        let int_x             = read_i32(44);
        let int_y             = read_i32(48);
        let extra_data_size   = read_u32(52) as usize;

        let extra_data = if flags.contains(PacketFlags::EXTENDED) {
            let start = GAME_PACKET_SIZE;
            let end   = start + extra_data_size;
            if data.len() < end {
                return None;
            }
            data[start..end].to_vec()
        } else {
            Vec::new()
        };

        Some(Self {
            packet_type,
            object_type,
            jump_count,
            animation_type,
            net_id,
            target_net_id,
            flags,
            float_variable,
            value,
            vector_x,
            vector_y,
            vector_x2,
            vector_y2,
            particle_rotation,
            int_x,
            int_y,
            extra_data,
        })
    }

    /// Serialize back to wire bytes (56-byte header + optional extra data).
    pub fn to_bytes(&self) -> Vec<u8> {
        let extra_data_size = if self.flags.contains(PacketFlags::EXTENDED) {
            self.extra_data.len()
        } else {
            0
        };

        let mut buf = vec![0u8; GAME_PACKET_SIZE + extra_data_size];

        buf[0] = self.packet_type.as_u8();
        buf[1] = self.object_type;
        buf[2] = self.jump_count;
        buf[3] = self.animation_type;
        buf[4..8].copy_from_slice(&self.net_id.to_le_bytes());
        buf[8..12].copy_from_slice(&self.target_net_id.to_le_bytes());
        buf[12..16].copy_from_slice(&self.flags.bits().to_le_bytes());
        buf[16..20].copy_from_slice(&self.float_variable.to_le_bytes());
        buf[20..24].copy_from_slice(&self.value.to_le_bytes());
        buf[24..28].copy_from_slice(&self.vector_x.to_le_bytes());
        buf[28..32].copy_from_slice(&self.vector_y.to_le_bytes());
        buf[32..36].copy_from_slice(&self.vector_x2.to_le_bytes());
        buf[36..40].copy_from_slice(&self.vector_y2.to_le_bytes());
        buf[40..44].copy_from_slice(&self.particle_rotation.to_le_bytes());
        buf[44..48].copy_from_slice(&self.int_x.to_le_bytes());
        buf[48..52].copy_from_slice(&self.int_y.to_le_bytes());
        buf[52..56].copy_from_slice(&(extra_data_size as u32).to_le_bytes());

        if extra_data_size > 0 {
            buf[GAME_PACKET_SIZE..].copy_from_slice(&self.extra_data);
        }

        buf
    }
}

impl fmt::Display for GameUpdatePacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GameUpdatePacket {{ type={:?}, net_id={}, pos=({:.1},{:.1}), vel=({:.1},{:.1}), flags={:?} }}",
            self.packet_type, self.net_id, self.vector_x, self.vector_y, self.vector_x2, self.vector_y2, self.flags
        )
    }
}

// ── Packet builders ──────────────────────────────────────────────────────────

/// Build an outer type-2 (text/action) packet payload ready for ENet.
pub fn make_text_packet(text: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + text.len() + 1);
    buf.extend_from_slice(&MSG_TEXT.to_le_bytes());
    buf.extend_from_slice(text.as_bytes());
    buf.push(0); // NUL terminator
    buf
}

/// Build an outer type-3 (game message) packet payload ready for ENet.
pub fn make_game_message_packet(text: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + text.len() + 1);
    buf.extend_from_slice(&MSG_GAME_MESSAGE.to_le_bytes());
    buf.extend_from_slice(text.as_bytes());
    buf.push(0); // NUL terminator
    buf
}

/// Build an outer type-4 (GameUpdatePacket) payload ready for ENet.
pub fn make_game_packet(pkt: &GameUpdatePacket) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + GAME_PACKET_SIZE);
    buf.extend_from_slice(&MSG_GAME_PACKET.to_le_bytes());
    buf.extend_from_slice(&pkt.to_bytes());
    buf
}

// ── Top-level packet dispatch ────────────────────────────────────────────────

#[derive(Debug)]
pub enum IncomingPacket<'a> {
    ServerHello,
    Text(&'a str),
    GameMessage(&'a str),
    GameUpdate(GameUpdatePacket),
    Track(&'a str),
    ClientLogRequest,
    Unknown { msg_type: u32, data: &'a [u8] },
}

impl<'a> IncomingPacket<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }
        let msg_type = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let payload  = &data[4..];

        match msg_type {
            MSG_SERVER_HELLO => Some(Self::ServerHello),
            MSG_TEXT | MSG_GAME_MESSAGE => {
                // NUL- or high-byte-terminated string
                let s = std::str::from_utf8(
                    payload.split(|&b| b == 0 || b >= 0x80).next().unwrap_or(payload)
                ).ok()?;
                if msg_type == MSG_TEXT {
                    Some(Self::Text(s))
                } else {
                    Some(Self::GameMessage(s))
                }
            }
            MSG_GAME_PACKET => {
                GameUpdatePacket::from_bytes(payload).map(Self::GameUpdate)
            }
            MSG_TRACK => {
                let s = std::str::from_utf8(
                    payload.split(|&b| b == 0 || b >= 0x80).next().unwrap_or(payload)
                ).ok()?;
                Some(Self::Track(s))
            }
            MSG_CLIENT_LOG_REQUEST => Some(Self::ClientLogRequest),
            other => Some(Self::Unknown { msg_type: other, data: payload }),
        }
    }
}
