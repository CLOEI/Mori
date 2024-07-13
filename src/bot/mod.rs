pub struct Bot {
    username: String,
    password: String,
    is_running: bool,
}

impl Bot {
    pub fn new(username: String, password: String) -> Bot {
        Bot {
            username,
            password,
            is_running: false,
        }
    }
}
