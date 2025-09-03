#[derive(Default, Debug, Clone)]
pub struct Player {
    pub _type: String,
    pub avatar: String,
    pub net_id: u32,
    pub online_id: String,
    pub e_id: String,
    pub ip: String,
    pub col_rect: String,
    pub title_icon: String,
    pub m_state: u32,
    pub user_id: u32,
    pub invisible: bool,
    pub name: String,
    pub country: String,
    pub position: (f32, f32),
}