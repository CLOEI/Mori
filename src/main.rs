mod bot;
mod manager;
mod types;
mod utils;
mod webserver;

use std::sync::Arc;

use manager::Manager;
use spdlog::prelude::*;
use types::e_login_method::ELoginMethod;

fn main() {
    let manager = match Manager::new() {
        Ok(manager) => manager,
        Err(err) => {
            error!("Error: {}", err);
            return;
        }
    };

    webserver::start(Arc::new(manager));
}
