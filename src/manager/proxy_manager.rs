use crate::{types, utils};
use socks::Socks5Datagram;
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Clone)]
pub struct Proxy {
    pub proxy: types::config::Proxy,
    pub status: String,
    pub whos_using: Vec<String>,
}

pub struct ProxyManager {
    pub proxies: Vec<Proxy>,
}

impl ProxyManager {
    pub fn new() -> Self {
        let proxies = utils::config::get_proxies();
        let new_proxies = proxies
            .iter()
            .map(|proxy| Proxy {
                proxy: proxy.clone(),
                status: "Not tested".to_string(),
                whos_using: vec![],
            })
            .collect();

        Self {
            proxies: new_proxies,
        }
    }
}

impl ProxyManager {
    pub fn add(&mut self, proxy: types::config::Proxy) {
        self.proxies.push(Proxy {
            proxy: proxy.clone(),
            status: "Not tested".to_string(),
            whos_using: vec![],
        });
        utils::config::add_proxy(proxy);
    }

    pub fn remove(&mut self, index: usize) {
        self.proxies.remove(index);
        utils::config::remove_proxy(index);
    }

    pub fn get(&self, index: usize) -> Option<&Proxy> {
        self.proxies.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Proxy> {
        self.proxies.get_mut(index)
    }

    pub fn test(&mut self, index: usize) {
        let mut proxy_data = self.proxies.get_mut(index).unwrap();
        let proxy = proxy_data.proxy.clone();
        let proxy_addr =
            SocketAddr::from_str(format!("{}:{}", proxy.ip.clone(), proxy.port.clone()).as_str())
                .expect("Invalid proxy address");
        if proxy.username.is_empty() {
            match Socks5Datagram::bind(proxy_addr, SocketAddr::from_str("0.0.0.0:0").unwrap()) {
                Ok(_) => {
                    proxy_data.status = "Functional".to_string();
                }
                Err(e) => {
                    proxy_data.status = "Not functional".to_string();
                }
            }
        } else {
            match Socks5Datagram::bind_with_password(
                proxy_addr,
                SocketAddr::from_str("0.0.0.0:0").unwrap(),
                &proxy.username,
                &proxy.password,
            ) {
                Ok(_) => {
                    proxy_data.status = "Functional".to_string();
                }
                Err(e) => {
                    proxy_data.status = "Not functional".to_string();
                }
            }
        }
    }
}
