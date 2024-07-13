use crate::bot::Bot;
use crate::types::e_login_method::ELoginMethod;

pub struct Manager {
    bots: Vec<Bot>,
}

impl Manager {
    pub fn new() -> Manager {
        Manager { bots: Vec::new() }
    }
}

pub trait BotManagement {
    fn add_bot(&mut self, username: &str, password: &str, method: ELoginMethod);
    fn remove_bot(&mut self, username: &str);
    fn get_bot(&self, username: &str);
}

impl BotManagement for Manager {
    fn add_bot(&mut self, username: &str, password: &str, method: ELoginMethod) {}
    fn remove_bot(&mut self, username: &str) {}
    fn get_bot(&self, username: &str) {}
}
