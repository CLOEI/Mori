use crate::manager::bot_manager::BotManager;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct Autofarm_c {
    pub selected_bot: String,
    pub is_farming: bool,
    pub is_paused: Arc<AtomicBool>,
    pub selected_positions: Vec<(i32, i32)>,
    pub item_id: u32,
    pub grid_selection: [[bool; 5]; 5],
    pub temp_item_id: String,
}

impl Autofarm_c {
    pub fn start_autofarm(&mut self, manager: Arc<RwLock<BotManager>>) {
        if self.selected_positions.is_empty() || self.is_farming {
            return;
        }

        self.is_farming = true;
        self.is_paused.store(false, Ordering::SeqCst);
        let positions = self.selected_positions.clone();
        let item_id = self.item_id;
        let bot_name = self.selected_bot.clone();
        let is_paused = Arc::clone(&self.is_paused);

        thread::spawn(move || {
            if let Some(bot) = manager.read().unwrap().get_bot(&bot_name).cloned() {
                for (x, y) in positions.iter().cycle() {
                    if is_paused.load(Ordering::SeqCst) {
                        break;
                    }

                    bot.place(*x, *y, item_id);
                    thread::sleep(Duration::from_millis(180));

                    bot.punch(*x, *y);
                    thread::sleep(Duration::from_millis(180));
                }
            }
        });
    }

    pub fn pause_autofarm(&mut self) {
        self.is_paused.store(true, Ordering::SeqCst);
        self.is_farming = false;
    }
}
