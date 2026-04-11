use std::collections::HashMap;

/// The bot's own identity in the current world.
#[derive(Debug, Default, Clone)]
pub struct LocalPlayer {
    pub net_id:       u32,
    pub user_id:      u32,
    /// From SetCharacterState: value (net_id to echo in ping reply).
    pub hack_type:    u32,
    /// From SetCharacterState: jump_count - 126 (tile X position).
    pub build_length: u8,
    /// From SetCharacterState: animation_type - 126 (tile Y position).
    pub punch_length: u8,
    /// From SetCharacterState: vector_x2 (gravity).
    pub gravity:      f32,
    /// From SetCharacterState: vector_y2 (velocity).
    pub velocity:     f32,
}

/// Another player present in the same world.
#[derive(Debug, Clone)]
pub struct Player {
    pub net_id:     u32,
    pub user_id:    u32,
    pub name:       String,
    pub country:    String,
    pub position:   (f32, f32),
    pub avatar:     String,
    pub online_id:  String,
    pub e_id:       String,
    pub ip:         String,
    pub col_rect:   String,
    pub title_icon: String,
    pub m_state:    u32,
    pub sm_state:   u32,
    pub invisible:  bool,
}

/// Parse a `key|value\nkey|value\n…` string into a map.
pub fn parse_pipe_map(s: &str) -> HashMap<String, String> {
    s.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '|');
            let key = parts.next()?.trim().to_string();
            let val = parts.next().unwrap_or("").to_string();
            if key.is_empty() { None } else { Some((key, val)) }
        })
        .collect()
}
