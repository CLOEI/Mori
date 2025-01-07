use crate::{manager::bot_manager::BotManager, texture_manager::TextureManager, utils};
use eframe::egui::{self, Ui};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct ItemDatabase {
    search_query: String,
    selected_item_index: Option<u32>,
}

impl ItemDatabase {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        manager: &Arc<RwLock<BotManager>>,
        texture_manager: &Arc<RwLock<TextureManager>>,
        _ctx: &egui::Context,
    ) {
        let manager = manager.read().unwrap();
        let items_database = manager.items_database.read().unwrap();

        if items_database.loaded {
            ui.horizontal(|ui| {
                let (items_database_version, items_database_item_count) =
                    { (items_database.version.clone(), items_database.item_count) };

                ui.label("Database version:");
                ui.label(items_database_version.to_string());
                ui.separator();
                ui.label("Items count:");
                ui.label(items_database_item_count.to_string());
                ui.separator();
                ui.text_edit_singleline(&mut self.search_query);
            });
            ui.separator();

            let mut filtered_items: Vec<u32> = {
                items_database
                    .items
                    .iter()
                    .filter_map(|(&id, item)| {
                        if item
                            .name
                            .to_lowercase()
                            .contains(&self.search_query.to_lowercase())
                        {
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
                    .id_salt("item_list_scroll_area")
                    .show_rows(ui, row_height, filtered_items.len(), |ui, row_range| {
                        ui.vertical(|ui| {
                            if row_range.is_empty() {
                                ui.add_sized([300.0, 0.0], egui::Label::new(""));
                            }
                            egui::Grid::new("item_database_grid")
                                .min_col_width(300.0)
                                .max_col_width(300.0)
                                .show(ui, |ui| {
                                    for i in row_range {
                                        let item_id = filtered_items[i];
                                        let item = { items_database.get_item(&item_id).unwrap() };
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
                    });

                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("item_info_scroll_area")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if let Some(selected_index) = self.selected_item_index {
                                let selected_item =
                                    { items_database.get_item(&selected_index).unwrap().clone() };
                                if selected_item.id % 2 == 0 {
                                    match texture_manager
                                        .read()
                                        .unwrap()
                                        .get_texture(&selected_item.texture_file_name)
                                    {
                                        Some(texture) => {
                                            let (spread_x, spread_y) =
                                                match selected_item.render_type {
                                                    2 | 5 => (4.0, 1.0),
                                                    4 => (4.0, 0.0),
                                                    3 | 7 | 8 | 9 | 10 => (3.0, 0.0),
                                                    _ => (0.0, 0.0),
                                                };

                                            let [width, height] = texture.size();
                                            let uv_x_start = ((selected_item.texture_x as f32
                                                + spread_x)
                                                * 32.0)
                                                / width as f32;
                                            let uv_y_start = ((selected_item.texture_y as f32
                                                + spread_y)
                                                * 32.0)
                                                / height as f32;
                                            let uv_x_end = (((selected_item.texture_x as f32
                                                + spread_x)
                                                * 32.0)
                                                + 32.0)
                                                / width as f32;
                                            let uv_y_end = (((selected_item.texture_y as f32
                                                + spread_y)
                                                * 32.0)
                                                + 32.0)
                                                / height as f32;

                                            let uv_start = egui::Pos2::new(uv_x_start, uv_y_start);
                                            let uv_end = egui::Pos2::new(uv_x_end, uv_y_end);

                                            ui.add(
                                                egui::Image::new(texture)
                                                    .uv(egui::Rect::from_min_max(uv_start, uv_end))
                                                    .fit_to_exact_size(egui::Vec2::new(32.0, 32.0)),
                                            );
                                        }
                                        None => (),
                                    }
                                } else {
                                    match texture_manager.read().unwrap().get_texture("seed.rttex")
                                    {
                                        Some(texture) => {
                                            let [width, height] = texture.size();
                                            let tile_size = 16.0;
                                            let scale_to_size = 32.0;

                                            // Calculate UV coordinates for the tile
                                            let tile_index = selected_item.seed_base_sprite as f32;
                                            let uv_x_start = (tile_index * 16.0) / width as f32;
                                            let uv_y_start = (0.0 * 16.0) / height as f32;
                                            let uv_x_end =
                                                ((tile_index * 16.0) + 16.0) / width as f32;
                                            let uv_y_end = ((0.0 * 16.0) + 16.0) / height as f32;

                                            let uv_rect = egui::Rect::from_min_max(
                                                egui::Pos2::new(uv_x_start, uv_y_start),
                                                egui::Pos2::new(uv_x_end, uv_y_end),
                                            );

                                            let (b, g, r, a) = utils::color::extract_bgra(
                                                selected_item.base_color,
                                            );
                                            let tint =
                                                egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                                            let (rect, painter) = ui.allocate_painter(
                                                egui::Vec2::new(scale_to_size, scale_to_size),
                                                egui::Sense::hover(),
                                            );

                                            painter.image(texture.id(), rect.rect, uv_rect, tint);

                                            let tile_index =
                                                selected_item.seed_overlay_sprite as f32;
                                            let uv_x_start = (tile_index * 16.0) / width as f32;
                                            let uv_y_start = (1.0 * 16.0) / height as f32;
                                            let uv_x_end =
                                                ((tile_index * 16.0) + 16.0) / width as f32;
                                            let uv_y_end = ((1.0 * 16.0) + 16.0) / height as f32;

                                            let uv_rect = egui::Rect::from_min_max(
                                                egui::Pos2::new(uv_x_start, uv_y_start),
                                                egui::Pos2::new(uv_x_end, uv_y_end),
                                            );

                                            let (b, g, r, a) = utils::color::extract_bgra(
                                                selected_item.overlay_color,
                                            );
                                            let tint =
                                                egui::Color32::from_rgba_unmultiplied(r, g, b, a);

                                            painter.image(texture.id(), rect.rect, uv_rect, tint);
                                        }
                                        None => (),
                                    }
                                }

                                ui.label(format!("Name: {}", selected_item.name));

                                if selected_item.id != 0 {
                                    ui.label(format!("ID: {}", selected_item.id));
                                }
                                // if selected_item.flags != 0 {
                                //     ui.label(format!("Flags: {}", selected_item.flags));
                                // }
                                if selected_item.action_type != 0 {
                                    ui.label(format!("Action Type: {}", selected_item.action_type));
                                }
                                ui.label(format!("Material: {}", selected_item.material));
                                if !selected_item.texture_file_name.is_empty() {
                                    ui.label(format!(
                                        "Texture File: {}",
                                        selected_item.texture_file_name
                                    ));
                                }
                                if selected_item.texture_hash != 0 {
                                    ui.label(format!(
                                        "Texture Hash: {}",
                                        selected_item.texture_hash
                                    ));
                                }
                                if selected_item.cooking_ingredient != 0 {
                                    ui.label(format!(
                                        "Cooking Ingredient: {}",
                                        selected_item.cooking_ingredient
                                    ));
                                }
                                ui.label(format!("Visual Effect: {}", selected_item.visual_effect));
                                ui.label(format!("Texture X: {}", selected_item.texture_x));
                                ui.label(format!("Texture Y: {}", selected_item.texture_y));
                                ui.label(format!("Render Type: {}", selected_item.render_type));
                                if selected_item.is_stripey_wallpaper != 0 {
                                    ui.label(format!(
                                        "Stripey Wallpaper: {}",
                                        selected_item.is_stripey_wallpaper
                                    ));
                                }
                                ui.label(format!(
                                    "Collision Type: {}",
                                    selected_item.collision_type
                                ));
                                if selected_item.block_health != 0 {
                                    ui.label(format!(
                                        "Block Health: {}",
                                        selected_item.block_health
                                    ));
                                }
                                if selected_item.drop_chance != 0 {
                                    ui.label(format!("Drop Chance: {}", selected_item.drop_chance));
                                }
                                ui.label(format!("Clothing Type: {}", selected_item.clothing_type));
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
                                    ui.label(format!(
                                        "Audio Volume: {}",
                                        selected_item.audio_volume
                                    ));
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
                                    ui.label(format!(
                                        "Seed Base Sprite: {}",
                                        selected_item.seed_base_sprite
                                    ));
                                }
                                if selected_item.seed_overlay_sprite != 0 {
                                    ui.label(format!(
                                        "Seed Overlay Sprite: {}",
                                        selected_item.seed_overlay_sprite
                                    ));
                                }
                                if selected_item.tree_base_sprite != 0 {
                                    ui.label(format!(
                                        "Tree Base Sprite: {}",
                                        selected_item.tree_base_sprite
                                    ));
                                }
                                if selected_item.tree_overlay_sprite != 0 {
                                    ui.label(format!(
                                        "Tree Overlay Sprite: {}",
                                        selected_item.tree_overlay_sprite
                                    ));
                                }
                                if selected_item.base_color != 0 {
                                    ui.label(format!("Base Color: {}", selected_item.base_color));
                                }
                                if selected_item.overlay_color != 0 {
                                    ui.label(format!(
                                        "Overlay Color: {}",
                                        selected_item.overlay_color
                                    ));
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
                                    ui.label(format!(
                                        "Extra Options: {}",
                                        selected_item.extra_options
                                    ));
                                }
                                if !selected_item.texture_path_2.is_empty() {
                                    ui.label(format!(
                                        "Texture Path 2: {}",
                                        selected_item.texture_path_2
                                    ));
                                }
                                if !selected_item.extra_option2.is_empty() {
                                    ui.label(format!(
                                        "Extra Option 2: {}",
                                        selected_item.extra_option2
                                    ));
                                }
                                if !selected_item.punch_option.is_empty() {
                                    ui.label(format!(
                                        "Punch Option: {}",
                                        selected_item.punch_option
                                    ));
                                }
                            } else {
                                ui.label("No item selected.");
                            }
                        });
                    });
            });
        } else {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add_space(ui.available_height() / 2.0 - 25.0);
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(egui_phosphor::variants::fill::DATABASE).size(50.0),
                        ));
                        ui.label("Database is not loaded. logon to the game to load the database.");
                    });
                    ui.add_space(ui.available_height() / 2.0 - 25.0);
                },
            );
        }
    }
}
