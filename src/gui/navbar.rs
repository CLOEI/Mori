use eframe::egui::{self, Ui};

pub struct Navbar {
    pub current_menu: String,
}

impl Default for Navbar {
    fn default() -> Self {
        Navbar {
            current_menu: "bots".to_string(),
        }
    }
}

impl Navbar {
    pub fn render(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(
                    egui_material_icons::icons::ICON_AUTO_AWESOME.to_owned() + " Bots",
                ))
                .clicked()
            {
                self.current_menu = "bots".to_string();
            }
            if ui
                .add(egui::Button::new(egui::RichText::new(
                    egui_material_icons::icons::ICON_DATABASE.to_owned() + " Item database",
                )))
                .clicked()
            {
                self.current_menu = "item_database".to_string();
            }
        });
    }
}
