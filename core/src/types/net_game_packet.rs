use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug, Clone)]
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

#[derive(Default, Debug)]
pub struct NetGamePacketData {
    pub _type: NetGamePacket,
    pub object_type: u8,
    pub jump_count: u8,
    pub animation_type: u8,
    pub net_id: u32,
    pub target_net_id: u32,
    pub flags: u32,
    pub float_variable: f32,
    pub value: u32,
    pub vector_x: f32,
    pub vector_y: f32,
    pub vector_x2: f32,
    pub vector_y2: f32,
    pub particle_rotation: f32,
    pub int_x: i32,
    pub int_y: i32,
    pub extended_data_length: u32,
}

impl NetGamePacketData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = vec![];
        wtr.push(self._type.clone() as u8);
        wtr.push(self.object_type);
        wtr.push(self.jump_count);
        wtr.push(self.animation_type);
        wtr.extend(&self.net_id.to_le_bytes());
        wtr.extend(&self.target_net_id.to_le_bytes());
        wtr.extend(&self.flags.to_le_bytes());
        wtr.extend(&self.float_variable.to_le_bytes());
        wtr.extend(&self.value.to_le_bytes());
        wtr.extend(&self.vector_x.to_le_bytes());
        wtr.extend(&self.vector_y.to_le_bytes());
        wtr.extend(&self.vector_x2.to_le_bytes());
        wtr.extend(&self.vector_y2.to_le_bytes());
        wtr.extend(&self.particle_rotation.to_le_bytes());
        wtr.extend(&self.int_x.to_le_bytes());
        wtr.extend(&self.int_y.to_le_bytes());
        wtr.extend(&self.extended_data_length.to_le_bytes());
        wtr
    }
    
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        let mut rdr = Cursor::new(data);
        if data.len() < std::mem::size_of::<NetGamePacketData>() {
            return None;
        }

        let _type = rdr.read_u8().ok().map(NetGamePacket::from)?;
        let object_type = rdr.read_u8().ok()?;
        let jump_count = rdr.read_u8().ok()?;
        let animation_type = rdr.read_u8().ok()?;
        let net_id = rdr.read_u32::<LittleEndian>().ok()?;
        let target_net_id = rdr.read_u32::<LittleEndian>().ok()?;
        let flags = rdr.read_u32::<LittleEndian>().ok()?;
        let float_variable = rdr.read_f32::<LittleEndian>().ok()?;
        let value = rdr.read_u32::<LittleEndian>().ok()?;
        let vector_x = rdr.read_f32::<LittleEndian>().ok()?;
        let vector_y = rdr.read_f32::<LittleEndian>().ok()?;
        let vector_x2 = rdr.read_f32::<LittleEndian>().ok()?;
        let vector_y2 = rdr.read_f32::<LittleEndian>().ok()?;
        let particle_rotation = rdr.read_f32::<LittleEndian>().ok()?;
        let int_x = rdr.read_i32::<LittleEndian>().ok()?;
        let int_y = rdr.read_i32::<LittleEndian>().ok()?;
        let extended_data_length = rdr.read_u32::<LittleEndian>().ok()?;

        Some(NetGamePacketData {
            _type,
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
            extended_data_length,
        })
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