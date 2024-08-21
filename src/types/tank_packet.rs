use super::etank_packet_type::ETankPacketType;

pub struct TankPacket {
    pub packet_type: ETankPacketType,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: u8,
    pub net_id: u32,
    pub unk4: u32,
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
