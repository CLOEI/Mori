use crate::types::etank_packet_type::ETankPacketType;
use crate::types::tank_packet::TankPacket;

pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
}

pub fn bytes_to_f32(bytes: &[u8]) -> f32 {
    f32::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
}

pub fn bytes_to_i32(bytes: &[u8]) -> i32 {
    i32::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
}

pub fn map_slice_to_tank_packet_type(data: &[u8]) -> TankPacket {
    TankPacket {
        _type: ETankPacketType::from(data[0]),
        unk1: data[1],
        unk2: data[2],
        unk3: data[3],
        net_id: bytes_to_u32(&data[4..8]),
        unk4: bytes_to_u32(&data[8..12]),
        flags: bytes_to_u32(&data[12..16]),
        unk6: bytes_to_f32(&data[16..20]),
        value: bytes_to_u32(&data[20..24]),
        vector_x: bytes_to_f32(&data[24..28]),
        vector_y: bytes_to_f32(&data[28..32]),
        vector_x2: bytes_to_f32(&data[32..36]),
        vector_y2: bytes_to_f32(&data[36..40]),
        unk12: bytes_to_f32(&data[40..44]),
        int_x: bytes_to_i32(&data[44..48]),
        int_y: bytes_to_i32(&data[48..52]),
        extended_data_length: bytes_to_u32(&data[52..56]),
    }
}