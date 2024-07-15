use super::e_tank_packet_type::ETankPacketType;

pub struct TankPacketType {
    pub packet_type: ETankPacketType,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: u8,
    pub net_id: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub unk7: u32,
    pub unk8: f32,
    pub unk9: f32,
    pub unk10: f32,
    pub unk11: f32,
    pub unk12: f32,
    pub unk13: u32,
    pub unk14: u32,
    pub extended_data_length: u32,
}

impl TankPacketType {
    pub fn new() -> TankPacketType {
        TankPacketType {
            packet_type: ETankPacketType::NetGamePacketCallFunction,
            unk1: 0,
            unk2: 0,
            unk3: 0,
            net_id: 0,
            unk4: 0,
            unk5: 0,
            unk6: 0,
            unk7: 0,
            unk8: 0.0,
            unk9: 0.0,
            unk10: 0.0,
            unk11: 0.0,
            unk12: 0.0,
            unk13: 0,
            unk14: 0,
            extended_data_length: 0,
        }
    }
}
