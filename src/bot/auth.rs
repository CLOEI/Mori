use crate::constants::{GAME_VER, PROTOCOL};
use crate::dashboard::get_dashboard_proxied;
use crate::login::{LoginError, get_legacy_token_proxied};
use crate::server_data::{LoginInfo, get_server_data_proxied};
use std::net::SocketAddr;

use super::shared::Socks5Config;

pub(super) struct Credentials {
    pub ltoken: String,
    pub meta: String,
    pub addr: SocketAddr,
}

pub(super) fn fetch_credentials(
    username: &str,
    password: &str,
    proxy: Option<&Socks5Config>,
    log: &mut dyn FnMut(String),
) -> Credentials {
    let proxy_url = proxy.map(|p| p.to_url());
    let proxy_url = proxy_url.as_deref();

    let login_info = LoginInfo {
        protocol: PROTOCOL,
        game_version: GAME_VER.into(),
    };

    let mut alternate = false;
    loop {
        log(format!(
            "[Bot] fetching server_data (alternate={alternate})..."
        ));
        let server_data = match get_server_data_proxied(alternate, &login_info, proxy_url) {
            Ok(s) => s,
            Err(e) => {
                alternate = !alternate;
                log(format!(
                    "[Bot] fetch: server_data failed: {e} - retrying in 5s"
                ));
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let dashboard = match get_dashboard_proxied(
            &server_data.loginurl,
            &login_info,
            &server_data.meta,
            proxy_url,
        ) {
            Ok(d) => d,
            Err(e) => {
                log(format!(
                    "[Bot] fetch: dashboard failed: {e} - retrying in 5s"
                ));
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let growtopia_url = match dashboard.growtopia {
            Some(u) => u,
            None => {
                log(format!(
                    "[Bot] fetch: no Growtopia URL in dashboard - retrying in 5s"
                ));
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let ltoken = match get_legacy_token_proxied(&growtopia_url, username, password, proxy_url)
        {
            Ok(t) => t,
            Err(e) => {
                log(format!("[Bot] fetch: login failed: {e}"));
                if matches!(e, LoginError::Exhausted) {
                    log("[Bot] login attempts exhausted - stopping".to_string());
                    panic!("[Bot] login attempts exhausted - stopping");
                }
                if matches!(e, LoginError::WrongCredentials) {
                    log("[Bot] wrong credentials - stopping".to_string());
                    panic!("[Bot] wrong credentials - stopping");
                }
                log("[Bot] retrying in 5s".to_string());
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let addr: SocketAddr = format!("{}:{}", server_data.server, server_data.port)
            .parse()
            .expect("Invalid server address");

        log(format!("[Bot] Got token: {ltoken}"));
        return Credentials {
            ltoken,
            meta: server_data.meta,
            addr,
        };
    }
}
