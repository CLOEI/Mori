use egui::{ColorImage, Context, TextureHandle};
use image::imageops::FilterType;
use image::DynamicImage;
use paris::info;
use std::collections::HashMap;
use std::{env, fs};
use std::path::Path;

pub struct TextureManager {
    pub textures: HashMap<String, TextureHandle>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn load_textures(&mut self, ctx: &Context, path: &Path) {
        let assets = fs::read_dir(path).expect("Failed to read assets directory");
        for asset in assets {
            let path = asset.unwrap().path();
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            if !filename.ends_with(".rttex") {
                continue;
            }

            let mut image_buffer =
                rttex::get_image_buffer(path.to_str().unwrap()).expect("Failed to load image");
            let size = [
                image_buffer.width() as usize,
                image_buffer.height() as usize,
            ];
            let pixels = image_buffer
                .pixels()
                .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                .collect();

            let image = ColorImage { size, pixels };

            let handler = ctx.load_texture(&filename, image, egui::TextureOptions::NEAREST);
            self.textures.insert(filename.clone(), handler);

            info!("Loaded texture: {}", filename);
        }
    }

    pub fn get_texture(&self, filename: &str) -> Option<&TextureHandle> {
        match self.textures.get(filename) {
            Some(texture) => Some(texture),
            None => None,
        }
    }

    pub fn reset_textures(&mut self) {
        self.textures.clear();
    }
}
