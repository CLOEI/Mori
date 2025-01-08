use crate::manager::bot_manager::BotManager;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use rand::Rng;

#[derive(Default, Clone)]
pub struct AutoSpam {
    pub selected_bot: String,
    pub is_talking: bool,
    pub is_enabled: bool,
    pub is_worldactive: bool,
    pub is_anti_positive_enabled: bool,
    pub is_paused: Arc<AtomicBool>,
    pub selected_xy_of_world: Vec<(i32, i32)>,
    pub temp_spam_text: String,
    pub temp_x_input: String,
    pub temp_y_input: String,
}

impl AutoSpam {
    pub fn start_autospam(&mut self, manager: Arc<RwLock<BotManager>>) {
        if self.selected_xy_of_world.is_empty() || self.is_talking || self.temp_spam_text.is_empty() {
            println!("Autospam started with errors: settings have been updated to 'zero'");
            return;
        }
        let positive = self.is_anti_positive_enabled;
        self.is_talking = true;
        self.is_paused.store(false, Ordering::SeqCst);

        let positions = self.selected_xy_of_world.clone();
        let spam_text = self.temp_spam_text.clone();
        let bot_name = self.selected_bot.clone();
        let is_paused = Arc::clone(&self.is_paused);

        let self_arc = Arc::new(RwLock::new(self.clone()));

        thread::spawn(move || {
            let mut rng = rand::thread_rng();

            if let Some(bot) = manager.read().unwrap().get_bot(&bot_name).cloned() {
                loop {
                    let player_count = {
                        let players = bot.players.lock().unwrap();
                        players.len()
                    };
                    if player_count == 1 && !self_arc.read().unwrap().is_worldactive {
                        self_arc.write().unwrap().is_worldactive = true;
                        println!("No players detected. Activating Spammer rest...");
                    } else if player_count > 0 && self_arc.read().unwrap().is_worldactive {
                        self_arc.write().unwrap().is_worldactive = false;
                        println!("Players detected. Stopping Resting...");
                        break;
                    }

                    if self_arc.read().unwrap().is_worldactive {
                        let positions = positions.clone();
                        let spam_text = spam_text.clone();
                        let is_paused = Arc::clone(&is_paused);

                        for (target_x, target_y) in positions.iter().cycle() {
                            if is_paused.load(Ordering::SeqCst) {
                                println!("Autospam paused.");
                                return;
                            }

                            let local = {
                                let position = bot.position.lock().unwrap();
                                position.clone()
                            };
                            let current_x = (local.x / 32.0) as i32;
                            let current_y = (local.y / 32.0) as i32;

                            if current_x != *target_x || current_y != *target_y {
                                println!("Travelling to target position: ({}, {})", target_x, target_y);
                                bot.find_path(*target_x as u32, *target_y as u32);
                            } else {
                                println!("Bot already in the right position: ({}, {})", target_x, target_y);
                            }

                            let modern_text = AutoSpam::randomize_text(&spam_text);

                            println!("Text spamming: {}", modern_text);
                            if bot.is_inworld() {
                                if !positive {
                                    bot.talk(spam_text.clone());
                                    let delay = rng.gen_range(4300..=5500);
                                    thread::sleep(Duration::from_millis(delay as u64));
                                } else {
                                    bot.talk(format!("{}", modern_text));
                                    let delay = rng.gen_range(4300..=5500);
                                    thread::sleep(Duration::from_millis(delay as u64));
                                }
                            } else {
                                println!("Autospam stopped with errors: settings have been updated to 'zero'");
                                return;
                            }


                            if !is_paused.load(Ordering::SeqCst) {
                                println!("Spam remaining.");
                            }
                        }
                    }
                }
            } else {
                println!("Bot '{}' cannot find in the list.", bot_name);
            }
        });
    }

    pub fn pause_autospam(&mut self) {
        println!("Autospam pausing...");
        self.is_paused.store(true, Ordering::SeqCst);
        self.is_talking = false;
    }

    pub fn add_position(&mut self, x: i32, y: i32) {
        self.selected_xy_of_world.push((x, y));
        println!("Added position: ({}, {})", x, y);
    }

    fn randomize_text(text: &str) -> String {
        let mut rng = rand::thread_rng();
        let mut chars: Vec<char> = text.chars().collect();
        let insertion_index = rng.gen_range(0..=chars.len());

        let mut result = String::new();
        for (i, c) in chars.iter().enumerate() {
            if i == insertion_index {
                result.push('`');
                result.push('`');
            }
            result.push(*c);
        }
        if insertion_index == chars.len() {
            result.push('`');
            result.push('`');
        }
        result
    }
}
