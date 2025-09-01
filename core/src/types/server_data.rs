use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ServerData {
    pub server: String,
    pub port: u16,
    pub loginurl: String,
    pub server_type: u8,
    pub beta_server: String,
    pub beta_loginurl: String,
    pub beta_port: u16,
    pub beta_type: u8,
    pub beta2_server: String,
    pub beta2_loginurl: String,
    pub beta2_port: u16,
    pub beta2_type: u8,
    pub beta3_server: String,
    pub beta3_loginurl: String,
    pub beta3_port: u16,
    pub beta3_type: u8,
    pub type2: u8,
    pub maint: Option<String>,
    pub meta: String,
}

impl ServerData {
    pub fn parse_from_response(response: &str) -> Result<Self> {
        let mut server_data = ServerData {
            server: String::new(),
            port: 0,
            loginurl: String::new(),
            server_type: 0,
            beta_server: String::new(),
            beta_loginurl: String::new(),
            beta_port: 0,
            beta_type: 0,
            beta2_server: String::new(),
            beta2_loginurl: String::new(),
            beta2_port: 0,
            beta2_type: 0,
            beta3_server: String::new(),
            beta3_loginurl: String::new(),
            beta3_port: 0,
            beta3_type: 0,
            type2: 0,
            maint: None,
            meta: String::new(),
        };

        for line in response.lines() {
            if line.starts_with("RTENDMARKERBS1001") {
                break;
            }

            if let Some((key, value)) = line.split_once('|') {
                match key {
                    "server" => server_data.server = value.to_string(),
                    "port" => server_data.port = value.parse()?,
                    "loginurl" => server_data.loginurl = value.to_string(),
                    "type" => server_data.server_type = value.parse()?,
                    "beta_server" => server_data.beta_server = value.to_string(),
                    "beta_loginurl" => server_data.beta_loginurl = value.to_string(),
                    "beta_port" => server_data.beta_port = value.parse()?,
                    "beta_type" => server_data.beta_type = value.parse()?,
                    "beta2_server" => server_data.beta2_server = value.to_string(),
                    "beta2_loginurl" => server_data.beta2_loginurl = value.to_string(),
                    "beta2_port" => server_data.beta2_port = value.parse()?,
                    "beta2_type" => server_data.beta2_type = value.parse()?,
                    "beta3_server" => server_data.beta3_server = value.to_string(),
                    "beta3_loginurl" => server_data.beta3_loginurl = value.to_string(),
                    "beta3_port" => server_data.beta3_port = value.parse()?,
                    "#maint" => server_data.maint = Some(value.to_string()),
                    "meta" => server_data.meta = value.to_string(),
                    _ => {} // Ignore unknown keys
                }
            }
        }

        Ok(server_data)
    }
}
