use std::sync::Arc;
use crate::manager::Manager;
use eframe::egui::{self, Ui};
use parking_lot::RwLock;

#[derive(Default)]
pub struct ItemDatabase {
    search_query: String,
    selected_item_index: Option<u32>,
}

impl ItemDatabase {
    pub fn render(&mut self, ui: &mut Ui, manager: &Arc<RwLock<Manager>>, _ctx: &egui::Context) {
        ui.horizontal(|ui| {
            let (item_database_version, item_database_item_count) = {
                let manager = manager.read();
                (manager.items_database.version.clone(), manager.items_database.item_count)
            };

            ui.label("Database version:");
            ui.label(item_database_version.to_string());
            ui.separator();
            ui.label("Items count:");
            ui.label(item_database_item_count.to_string());
            ui.separator();
            ui.text_edit_singleline(&mut self.search_query);
        });
        ui.separator();

        let mut filtered_items: Vec<u32> = {
            manager.read().items_database.items.iter()
                .filter_map(|(&id, item)| {
                    if item.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect()
        };

        filtered_items.sort();

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            let text_style = egui::TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);

            egui::ScrollArea::vertical()
                .id_source("item_list_scroll_area")
                .show_rows(
                    ui,
                    row_height,
                    filtered_items.len(),
                    |ui, row_range| {
                        ui.vertical(|ui| {
                            egui::Grid::new("item_database_grid")
                                .min_col_width(300.0)
                                .max_col_width(300.0)
                                .show(ui, |ui| {
                                    for i in row_range {
                                        let item_id = filtered_items[i];
                                        let item = {
                                            let manager = manager.read();
                                            manager.items_database.get_item(&item_id).unwrap()
                                        };
                                        if ui
                                            .selectable_label(
                                                self.selected_item_index == Some(item_id),
                                                &item.name,
                                            )
                                            .clicked()
                                        {
                                            self.selected_item_index = Some(item_id);
                                        }
                                        ui.end_row();
                                    }
                                });
                        })
                    },
                );

            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("item_info_scroll_area")
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        if let Some(selected_index) = self.selected_item_index {
                            let selected_item = {
                                let manager = manager.read();
                                manager.items_database.get_item(&selected_index).unwrap().clone()
                            };
                            ui.label(format!("Name: {}", selected_item.name));

                            if selected_item.id != 0 {
                                ui.label(format!("ID: {}", selected_item.id));
                            }
                            if selected_item.flags != 0 {
                                ui.label(format!("Flags: {}", selected_item.flags));
                            }
                            if selected_item.action_type != 0 {
                                ui.label(format!("Action Type: {}", selected_item.action_type));
                            }
                            if selected_item.material != 0 {
                                ui.label(format!("Material: {}", selected_item.material));
                            }
                            if !selected_item.texture_file_name.is_empty() {
                                ui.label(format!("Texture File: {}", selected_item.texture_file_name));
                            }
                            if selected_item.texture_hash != 0 {
                                ui.label(format!("Texture Hash: {}", selected_item.texture_hash));
                            }
                            if selected_item.cooking_ingredient != 0 {
                                ui.label(format!("Cooking Ingredient: {}", selected_item.cooking_ingredient));
                            }
                            if selected_item.visual_effect != 0 {
                                ui.label(format!("Visual Effect: {}", selected_item.visual_effect));
                            }
                            if selected_item.texture_x != 0 {
                                ui.label(format!("Texture X: {}", selected_item.texture_x));
                            }
                            if selected_item.texture_y != 0 {
                                ui.label(format!("Texture Y: {}", selected_item.texture_y));
                            }
                            if selected_item.render_type != 0 {
                                ui.label(format!("Render Type: {}", selected_item.render_type));
                            }
                            if selected_item.is_stripey_wallpaper != 0 {
                                ui.label(format!("Stripey Wallpaper: {}", selected_item.is_stripey_wallpaper));
                            }
                            if selected_item.collision_type != 0 {
                                ui.label(format!("Collision Type: {}", selected_item.collision_type));
                            }
                            if selected_item.block_health != 0 {
                                ui.label(format!("Block Health: {}", selected_item.block_health));
                            }
                            if selected_item.drop_chance != 0 {
                                ui.label(format!("Drop Chance: {}", selected_item.drop_chance));
                            }
                            if selected_item.clothing_type != 0 {
                                ui.label(format!("Clothing Type: {}", selected_item.clothing_type));
                            }
                            if selected_item.rarity != 0 {
                                ui.label(format!("Rarity: {}", selected_item.rarity));
                            }
                            if selected_item.max_item != 0 {
                                ui.label(format!("Max Item: {}", selected_item.max_item));
                            }
                            if !selected_item.file_name.is_empty() {
                                ui.label(format!("File Name: {}", selected_item.file_name));
                            }
                            if selected_item.file_hash != 0 {
                                ui.label(format!("File Hash: {}", selected_item.file_hash));
                            }
                            if selected_item.audio_volume != 0 {
                                ui.label(format!("Audio Volume: {}", selected_item.audio_volume));
                            }
                            if !selected_item.pet_name.is_empty() {
                                ui.label(format!("Pet Name: {}", selected_item.pet_name));
                            }
                            if !selected_item.pet_prefix.is_empty() {
                                ui.label(format!("Pet Prefix: {}", selected_item.pet_prefix));
                            }
                            if !selected_item.pet_suffix.is_empty() {
                                ui.label(format!("Pet Suffix: {}", selected_item.pet_suffix));
                            }
                            if !selected_item.pet_ability.is_empty() {
                                ui.label(format!("Pet Ability: {}", selected_item.pet_ability));
                            }
                            if selected_item.seed_base_sprite != 0 {
                                ui.label(format!("Seed Base Sprite: {}", selected_item.seed_base_sprite));
                            }
                            if selected_item.seed_overlay_sprite != 0 {
                                ui.label(format!("Seed Overlay Sprite: {}", selected_item.seed_overlay_sprite));
                            }
                            if selected_item.tree_base_sprite != 0 {
                                ui.label(format!("Tree Base Sprite: {}", selected_item.tree_base_sprite));
                            }
                            if selected_item.tree_overlay_sprite != 0 {
                                ui.label(format!("Tree Overlay Sprite: {}", selected_item.tree_overlay_sprite));
                            }
                            if selected_item.base_color != 0 {
                                ui.label(format!("Base Color: {}", selected_item.base_color));
                            }
                            if selected_item.overlay_color != 0 {
                                ui.label(format!("Overlay Color: {}", selected_item.overlay_color));
                            }
                            if selected_item.ingredient != 0 {
                                ui.label(format!("Ingredient: {}", selected_item.ingredient));
                            }
                            if selected_item.grow_time != 0 {
                                ui.label(format!("Grow Time: {}", selected_item.grow_time));
                            }
                            if selected_item.is_rayman != 0 {
                                ui.label(format!("Is Rayman: {}", selected_item.is_rayman));
                            }
                            if !selected_item.extra_options.is_empty() {
                                ui.label(format!("Extra Options: {}", selected_item.extra_options));
                            }
                            if !selected_item.texture_path_2.is_empty() {
                                ui.label(format!("Texture Path 2: {}", selected_item.texture_path_2));
                            }
                            if !selected_item.extra_option2.is_empty() {
                                ui.label(format!("Extra Option 2: {}", selected_item.extra_option2));
                            }
                            if !selected_item.punch_option.is_empty() {
                                ui.label(format!("Punch Option: {}", selected_item.punch_option));
                            }
                        } else {
                            ui.label("No item selected.");
                        }
                    });
                });
        });
    }
}

