use crate::connect::Connect;
use crate::types::e_login_method::ELoginMethod;

pub struct Bot {
    username: String,
    password: String,
    method: ELoginMethod,
    oauth_links: Vec<String>,
    is_running: bool,
    connect: Connect,
}

impl Bot {
    pub fn new(
        username: String,
        password: String,
        method: ELoginMethod,
        oauth_links: Vec<String>,
    ) -> Bot {
        Bot {
            username,
            password,
            method,
            oauth_links,
            is_running: false,
            connect: Connect::new(),
        }
    }
}

impl Bot {
    pub fn start(&mut self) {
        match self.method {
            ELoginMethod::APPLE => {
                self.connect.get_apple_token(
                    self.oauth_links[0].as_str(),
                    self.username.as_str(),
                    self.password.as_str(),
                );
            }
            ELoginMethod::GOOGLE => {
                self.connect.get_google_token(
                    self.oauth_links[1].as_str(),
                    self.username.as_str(),
                    self.password.as_str(),
                );
            }
            ELoginMethod::LEGACY => {
                let _ = self.connect.get_legacy_token(
                    self.oauth_links[2].as_str(),
                    self.username.as_str(),
                    self.password.as_str(),
                );
            }
        }
    }

    fn stop(&mut self) {
        self.is_running = false;
    }
}
