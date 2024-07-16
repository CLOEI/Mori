use super::e_tank_packet_type::ETankPacketType;

pub struct TankPacketType {
    pub packet_type: ETankPacketType,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: u8,
    pub net_id: u32,
    pub unk4: u32,
    pub flags: u32,
    pub unk6: u32,
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

impl TankPacketType {
    pub fn new() -> TankPacketType {
        TankPacketType {
            packet_type: ETankPacketType::NetGamePacketCallFunction,
            unk1: 0,
            unk2: 0,
            unk3: 0,
            net_id: 0,
            unk4: 0,
            flags: 0,
            unk6: 0,
            value: 0,
            vector_x: 0.0,
            vector_y: 0.0,
            vector_x2: 0.0,
            vector_y2: 0.0,
            unk12: 0.0,
            int_x: 0,
            int_y: 0,
            extended_data_length: 0,
        }
    }
}
