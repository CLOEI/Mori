pub fn bytes_to_u32(slice: &[u8]) -> u32 {
    u32::from_le_bytes(slice.try_into().expect("slice with incorrect length"))
}

pub fn bytes_to_f32(slice: &[u8]) -> f32 {
    f32::from_le_bytes(slice.try_into().expect("slice with incorrect length"))
}
