use crate::manager::bot_manager::BotManager;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct Autospamv1 {
    pub selected_bot: String,
    pub is_talking: bool,
    pub is_paused: Arc<AtomicBool>,
    pub selected_xy_of_world: Vec<(i32, i32)>,
    pub temp_spam_text: String,
    pub temp_x_input: String,
    pub temp_y_input: String,

}

impl Autospamv1 {
    pub fn start_autospam(&mut self, manager: Arc<RwLock<BotManager>>) {
        if self.selected_xy_of_world.is_empty() || self.is_talking || self.temp_spam_text.is_empty() {
            println!("Autospam started with errors: settings have been updated to 'zero'");
            return;
        }

        self.is_talking = true;
        self.is_paused.store(false, Ordering::SeqCst);

        let positions = self.selected_xy_of_world.clone();
        let spam_text = self.temp_spam_text.clone();
        let bot_name = self.selected_bot.clone();
        let is_paused = Arc::clone(&self.is_paused);

        thread::spawn(move || {
            if let Some(bot) = manager.read().unwrap().get_bot(&bot_name).cloned() {
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

                    println!("Text spamming: {}", spam_text);
                    bot.talk(spam_text.clone());
                    thread::sleep(Duration::from_secs(5));
                    if !is_paused.load(Ordering::SeqCst) {
                        println!("Spam remaining.");
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
}