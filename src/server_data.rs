use std::time::Duration;
use ureq::tls::TlsConfig;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct LoginInfo {
    pub protocol: u32,
    pub game_version: String,
}

impl LoginInfo {
    pub fn to_form_data(&self) -> String {
        format!("protocol={}&version={}", self.protocol, self.game_version)
    }
}

#[derive(Debug, Default)]
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
        let mut data = ServerData::default();

        for line in response.lines() {
            if line.starts_with("RTENDMARKERBS1001") {
                break;
            }
            let Some((key, value)) = line.split_once('|') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "server" => data.server = value.into(),
                "port" => data.port = value.parse()?,
                "loginurl" => data.loginurl = value.into(),
                "type" => data.server_type = value.parse()?,
                "beta_server" => data.beta_server = value.into(),
                "beta_loginurl" => data.beta_loginurl = value.into(),
                "beta_port" => data.beta_port = value.parse()?,
                "beta_type" => data.beta_type = value.parse()?,
                "beta2_server" => data.beta2_server = value.into(),
                "beta2_loginurl" => data.beta2_loginurl = value.into(),
                "beta2_port" => data.beta2_port = value.parse()?,
                "beta2_type" => data.beta2_type = value.parse()?,
                "beta3_server" => data.beta3_server = value.into(),
                "beta3_loginurl" => data.beta3_loginurl = value.into(),
                "beta3_port" => data.beta3_port = value.parse()?,
                "beta3_type" => data.beta3_type = value.parse()?,
                "type2" => data.type2 = value.parse()?,
                "#maint" => data.maint = Some(value.into()),
                "meta" => data.meta = value.into(),
                _ => {}
            }
        }

        Ok(data)
    }
}

pub fn get_server_data(alternate: bool, login_info: &LoginInfo) -> Result<ServerData> {
    get_server_data_proxied(alternate, login_info, None)
}

pub fn get_server_data_proxied(
    alternate: bool,
    login_info: &LoginInfo,
    proxy_url: Option<&str>,
) -> Result<ServerData> {
    let url = if alternate {
        "https://www.growtopia2.com/growtopia/server_data.php"
    } else {
        "https://www.growtopia1.com/growtopia/server_data.php"
    };

    println!("[server_data] proxy_url={:?}", proxy_url);
    let agent = if let Some(p) = proxy_url {
        let proxy = ureq::Proxy::new(p)?;
        ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .proxy(Some(proxy))
                .tls_config(TlsConfig::builder().disable_verification(true).build())
                .timeout_global(Some(Duration::from_secs(20)))
                .build(),
        )
    } else {
        ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .tls_config(TlsConfig::builder().disable_verification(true).build())
                .timeout_global(Some(Duration::from_secs(20)))
                .build(),
        )
    };
    let body = agent
        .post(url)
        .header(
            "User-Agent",
            "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
        )
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(format!(
            "platform=0&protocol={}&version={}",
            login_info.protocol, login_info.game_version
        ))?
        .body_mut()
        .read_to_string()?;

    ServerData::parse_from_response(&body)
}
