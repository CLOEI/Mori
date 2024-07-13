mod bot;
mod connect;
mod manager;
mod types;

use manager::Manager;
use spdlog::prelude::*;
use types::e_login_method::ELoginMethod;

fn main() {
    let mut manager = match Manager::new() {
        Ok(manager) => manager,
        Err(err) => {
            error!("Error: {}", err);
            std::process::exit(1);
        }
    };
    manager.add_bot("Peroperod", "", ELoginMethod::LEGACY);
}
