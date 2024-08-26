use super::vector::Vector2;

#[derive(Default, Debug, Clone)]
pub struct Player {
    pub _type: String,
    pub avatar: String,
    pub net_id: u32,
    pub online_id: String,
    pub e_id: String,
    pub ip: String,
    pub colrect: String,
    pub title_icon: String,
    pub mstate: u32,
    pub user_id: u32,
    pub invis: bool,
    pub name: String,
    pub country: String,
    pub position: Vector2,
}
