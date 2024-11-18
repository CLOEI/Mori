pub fn extract_bgra(color: u32) -> (u8, u8, u8, u8) {
    let b = (color >> 24) as u8;
    let g = ((color >> 16) & 0xFF) as u8;
    let r = ((color >> 8) & 0xFF) as u8;
    let a = (color & 0xFF) as u8;
    (b, g, r, a)
}
