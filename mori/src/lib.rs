use godot::classes::{ISprite2D, Sprite2D};
use godot::prelude::*;

struct MoriExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MoriExtension {}

#[derive(GodotClass)]
#[class(base=Sprite2D)]
struct World {
    base: Base<Sprite2D>
}

#[godot_api]
impl ISprite2D for World {
    fn init(base: Base<Sprite2D>) -> Self {
        Self {
            base,
        }
    }
}