use rusty_enet as enet;
use serde::Serialize;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::bot::Socks5Config;
use crate::constants::{GAME_VER, PROTOCOL};
use crate::server_data::{LoginInfo, get_server_data_proxied};
use crate::socks5::Socks5UdpSocket;

#[derive(Serialize)]
pub struct CheckResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Serialize)]
pub struct ProxyTestResult {
    pub socks5: CheckResult,
    pub server_data: CheckResult,
    pub enet: CheckResult,
}

pub fn run_proxy_test(cfg: Socks5Config) -> ProxyTestResult {
    let login_info = LoginInfo {
        protocol: PROTOCOL,
        game_version: GAME_VER.into(),
    };
    let proxy_url = cfg.to_url();

    let socks5_check = {
        let local: SocketAddr = "0.0.0.0:0".parse().unwrap();
        match Socks5UdpSocket::bind_through_proxy(
            local,
            cfg.proxy_addr,
            cfg.username.as_deref(),
            cfg.password.as_deref(),
        ) {
            Ok(_) => CheckResult {
                ok: true,
                error: None,
                detail: None,
            },
            Err(e) => CheckResult {
                ok: false,
                error: Some(e.to_string()),
                detail: None,
            },
        }
    };

    let (server_data_check, server_addr) = {
        let result = get_server_data_proxied(false, &login_info, Some(&proxy_url))
            .or_else(|_| get_server_data_proxied(true, &login_info, Some(&proxy_url)));
        match result {
            Ok(sd) => {
                let detail = format!("{}:{}", sd.server, sd.port);
                let addr = format!("{}:{}", sd.server, sd.port)
                    .parse::<SocketAddr>()
                    .ok();
                (
                    CheckResult {
                        ok: true,
                        error: None,
                        detail: Some(detail),
                    },
                    addr,
                )
            }
            Err(e) => (
                CheckResult {
                    ok: false,
                    error: Some(e.to_string()),
                    detail: None,
                },
                None,
            ),
        }
    };

    let enet_check = match server_addr {
        None => CheckResult {
            ok: false,
            error: Some("skipped: server_data check failed".into()),
            detail: None,
        },
        Some(addr) => check_enet(&cfg, addr),
    };

    ProxyTestResult {
        socks5: socks5_check,
        server_data: server_data_check,
        enet: enet_check,
    }
}

fn check_enet(cfg: &Socks5Config, server_addr: SocketAddr) -> CheckResult {
    let local: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let socket = match Socks5UdpSocket::bind_through_proxy(
        local,
        cfg.proxy_addr,
        cfg.username.as_deref(),
        cfg.password.as_deref(),
    ) {
        Ok(s) => s,
        Err(e) => {
            return CheckResult {
                ok: false,
                error: Some(e.to_string()),
                detail: None,
            };
        }
    };

    let settings = enet::HostSettings {
        peer_limit: 1,
        channel_limit: 2,
        compressor: Some(Box::new(enet::RangeCoder::new())),
        checksum: Some(Box::new(enet::crc32)),
        using_new_packet: true,
        ..Default::default()
    };

    let mut host = match enet::Host::new(socket, settings) {
        Ok(h) => h,
        Err(e) => {
            return CheckResult {
                ok: false,
                error: Some(e.to_string()),
                detail: None,
            };
        }
    };

    if let Err(e) = host.connect(server_addr, 2, 0) {
        return CheckResult {
            ok: false,
            error: Some(e.to_string()),
            detail: None,
        };
    }

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if Instant::now() >= deadline {
            return CheckResult {
                ok: false,
                error: Some("timed out waiting for ENet connect".into()),
                detail: None,
            };
        }
        match host.service() {
            Ok(Some(event)) => {
                if let enet::EventNoRef::Connect { .. } = event.no_ref() {
                    return CheckResult {
                        ok: true,
                        error: None,
                        detail: None,
                    };
                }
            }
            Ok(None) => {}
            Err(e) => {
                return CheckResult {
                    ok: false,
                    error: Some(e.to_string()),
                    detail: None,
                };
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
