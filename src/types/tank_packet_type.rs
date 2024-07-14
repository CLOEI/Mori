use super::e_tank_packet_type::ETankPacketType;

pub struct TankPacketType {
    packet_type: ETankPacketType,
    unk1: u8,
    unk2: u8,
    unk3: u8,
    net_id: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
    unk7: u32,
    unk8: f32,
    unk9: f32,
    unk10: f32,
    unk11: f32,
    unk12: f32,
    unk13: u32,
    unk14: u32,
    extended_data_length: u32,
}
