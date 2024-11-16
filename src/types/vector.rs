#[derive(Default, Debug, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }
}
