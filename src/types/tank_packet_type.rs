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
