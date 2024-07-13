mod bot;
mod connect;
mod manager;
mod types;

use manager::{BotManagement, Manager};
use types::e_login_method::ELoginMethod;

fn main() {
    let mut manager = Manager::new();
    manager.add_bot("Peroperod", "", ELoginMethod::LEGACY);
    connect::get_oauth_links();
}
