use crate::manager::Manager;
use eframe::egui::{self, Ui};

#[derive(Default)]
pub struct ItemDatabase {
    search_query: String,
}

impl ItemDatabase {
    pub fn render(&mut self, ui: &mut Ui, manager: &mut Manager, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("Database version:");
            ui.label(manager.items_database.version.to_string());
            ui.separator();
            ui.label("Items count:");
            ui.label(manager.items_database.item_count.to_string());
            ui.separator();
            ui.text_edit_singleline::<String>(&mut self.search_query);
        });
        ui.separator();
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            let text_style = egui::TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            egui::ScrollArea::vertical().show_rows(
                ui,
                row_height,
                (manager.items_database.item_count) as usize,
                |ui, row_range| {
                    ui.vertical(|ui| {
                        egui::Grid::new("item_database_grid")
                            .min_col_width(300.0)
                            .max_col_width(300.0)
                            .show(ui, |ui| {
                                for i in row_range {
                                    ui.label(
                                        manager.items_database.items[&(i as u32)].name.clone(),
                                    );
                                    ui.end_row();
                                }
                            });
                    })
                },
            );
            ui.separator();
            ui.vertical(|ui| {
                ui.label("Item info");
            });
        });
    }
}
