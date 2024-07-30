use eframe::egui::{self};

use crate::manager::Manager;

#[derive(Default)]
pub struct WarpDialog {
    pub open: bool,
    pub name: String,
}

impl WarpDialog {
    pub fn render(&mut self, selected_bot: &str, manager: &mut Manager, ctx: &egui::Context) {
        if self.open {
            let mut close_dialog = false;
            egui::Window::new("Warp")
                .resizable(false)
                .open(&mut self.open)
                .show(ctx, |ui| {
                    egui::Grid::new("warp_grid")
                        .min_col_width(100.0)
                        .max_col_width(100.0)
                        .show(ui, |ui| {
                            ui.label("Name");
                            ui.text_edit_singleline(&mut self.name);
                            ui.end_row();
                        });
                    if ui.button("Warp").clicked() {
                        close_dialog = true;
                    }
                });
            if close_dialog {
                self.open = false;
            }
        }
    }
}
