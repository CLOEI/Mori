use std::{mem, ptr};

#[derive(Debug)]
#[repr(u8)]
pub enum NetGamePacket {
    State,
    CallFunction,
    UpdateStatus,
    TileChangeRequest,
    SendMapData,
    SendTileUpdateData,
    SendTileUpdateDataMultiple,
    TileActivateRequest,
    TileApplyDamage,
    SendInventoryState,
    ItemActivateRequest,
    ItemActivateObjectRequest,
    SendTileTreeState,
    ModifyItemInventory,
    ItemChangeObject,
    SendLock,
    SendItemDatabaseData,
    SendParticleEffect,
    SetIconState,
    ItemEffect,
    SetCharacterState,
    PingReply,
    PingRequest,
    GotPunched,
    AppCheckResponse,
    AppIntegrityFail,
    Disconnect,
    BattleJoin,
    BattleEvent,
    UseDoor,
    SendParental,
    GoneFishin,
    Steam,
    PetBattle,
    Npc,
    Special,
    SendParticleEffectV2,
    ActivateArrowToItem,
    SelectTileIndex,
    SendPlayerTributeData,
    FTUESetItemToQuickInventory,
    PVENpc,
    PVPCardBattle,
    PVEApplyPlayerDamage,
    PVENPCPositionUpdate,
    SetExtraMods,
    OnStepTileMod,
}

#[derive(Default)]
#[repr(packed, C)]
pub struct NetGamePacketData {
    pub _type: NetGamePacket,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: u8,
    pub net_id: u32,
    pub sec_id: u32,
    pub flags: u32,
    pub unk6: f32,
    pub value: u32,
    pub vector_x: f32,
    pub vector_y: f32,
    pub vector_x2: f32,
    pub vector_y2: f32,
    pub unk12: f32,
    pub int_x: i32,
    pub int_y: i32,
    pub extended_data_length: u32,
}

impl NetGamePacketData {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < mem::size_of::<Self>() {
            return None;
        }

        // TODO: do not use unsafe
        unsafe {
            let pointer = data.as_ptr() as *const Self;
            Some(ptr::read_unaligned(pointer))
        }
    }
}

impl Default for NetGamePacket {
    fn default() -> Self {
        NetGamePacket::State
    }
}

impl From<u8> for NetGamePacket {
    fn from(value: u8) -> Self {
        match value {
            0 => NetGamePacket::State,
            1 => NetGamePacket::CallFunction,
            2 => NetGamePacket::UpdateStatus,
            3 => NetGamePacket::TileChangeRequest,
            4 => NetGamePacket::SendMapData,
            5 => NetGamePacket::SendTileUpdateData,
            6 => NetGamePacket::SendTileUpdateDataMultiple,
            7 => NetGamePacket::TileActivateRequest,
            8 => NetGamePacket::TileApplyDamage,
            9 => NetGamePacket::SendInventoryState,
            10 => NetGamePacket::ItemActivateRequest,
            11 => NetGamePacket::ItemActivateObjectRequest,
            12 => NetGamePacket::SendTileTreeState,
            13 => NetGamePacket::ModifyItemInventory,
            14 => NetGamePacket::ItemChangeObject,
            15 => NetGamePacket::SendLock,
            16 => NetGamePacket::SendItemDatabaseData,
            17 => NetGamePacket::SendParticleEffect,
            18 => NetGamePacket::SetIconState,
            19 => NetGamePacket::ItemEffect,
            20 => NetGamePacket::SetCharacterState,
            21 => NetGamePacket::PingReply,
            22 => NetGamePacket::PingRequest,
            23 => NetGamePacket::GotPunched,
            24 => NetGamePacket::AppCheckResponse,
            25 => NetGamePacket::AppIntegrityFail,
            26 => NetGamePacket::Disconnect,
            27 => NetGamePacket::BattleJoin,
            28 => NetGamePacket::BattleEvent,
            29 => NetGamePacket::UseDoor,
            30 => NetGamePacket::SendParental,
            31 => NetGamePacket::GoneFishin,
            32 => NetGamePacket::Steam,
            33 => NetGamePacket::PetBattle,
            34 => NetGamePacket::Npc,
            35 => NetGamePacket::Special,
            36 => NetGamePacket::SendParticleEffectV2,
            37 => NetGamePacket::ActivateArrowToItem,
            38 => NetGamePacket::SelectTileIndex,
            39 => NetGamePacket::SendPlayerTributeData,
            40 => NetGamePacket::FTUESetItemToQuickInventory,
            41 => NetGamePacket::PVENpc,
            42 => NetGamePacket::PVPCardBattle,
            43 => NetGamePacket::PVEApplyPlayerDamage,
            44 => NetGamePacket::PVENPCPositionUpdate,
            45 => NetGamePacket::SetExtraMods,
            46 => NetGamePacket::OnStepTileMod,
            _ => NetGamePacket::State,
        }
    }
}