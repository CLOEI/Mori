use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use enet::{Address, BandwidthLimit, ChannelLimit, Enet, Event, Host};

use crate::types::{
    bot_info::{Info, Server, State},
    elogin_method::ELoginMethod,
    login_info::LoginInfo,
    vector::Vector2,
};

pub struct Bot<'a> {
    pub info: Info,
    pub state: State,
    pub server: Server,
    pub position: Vector2,
    pub host: Host<()>,
    pub peer: Option<enet::Peer<'a, ()>>,
}

impl<'a> Bot<'a> {
    pub fn new(
        username: String,
        password: String,
        recovery_code: String,
        login_method: ELoginMethod,
    ) -> Self {
        let enet = Enet::new().expect("could not initialize ENet");
        let host = enet
            .create_host::<()>(
                None,
                1,
                ChannelLimit::Limited(2),
                BandwidthLimit::Unlimited,
                BandwidthLimit::Unlimited,
                true,
            )
            .expect("could not create host");

        Self {
            info: Info {
                username,
                password,
                recovery_code,
                login_method,
                login_info: LoginInfo::new(),
                ..Default::default()
            },
            state: State::default(),
            server: Server::default(),
            position: Vector2::default(),
            host: host,
            peer: None,
        }
    }
}

pub fn logon(bot_arc: &Arc<Mutex<Bot>>) {
    let (ip, port) = {
        let bot = bot_arc.lock().unwrap();
        (
            bot.info.server_data.get("server").unwrap().clone(),
            bot.info.server_data.get("port").unwrap().clone(),
        )
    };

    to_http(bot_arc);
    connect_to_server(bot_arc, ip, port);
    process_events(bot_arc);
}

pub fn to_http(bot_arc: &Arc<Mutex<Bot>>) {
    let req = ureq::post("https://www.growtopia1.com/growtopia/server_data.php").set(
        "User-Agent",
        "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
    );

    let res = req.send_string("").unwrap();
    let body = res.into_string().unwrap();
    parse_server_data(bot_arc, body);
}

pub fn parse_server_data(bot_arc: &Arc<Mutex<Bot>>, data: String) {
    let mut bot = bot_arc.lock().unwrap();
    bot.info.server_data = data
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '|');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                _ => None,
            }
        })
        .collect::<HashMap<String, String>>();
}

fn connect_to_server(bot_arc: &Arc<Mutex<Bot>>, ip: String, port: String) {
    let mut bot = bot_arc.lock().unwrap();
    bot.host
        .borrow_mut()
        .connect(
            &Address::new(ip.parse().unwrap(), port.parse().unwrap()),
            2,
            0,
        )
        .expect("Failed to connect to the server");
}

fn process_events(bot_arc: &Arc<Mutex<Bot>>) {
    loop {
        match bot_arc
            .lock()
            .unwrap()
            .host
            .service(1000)
            .expect("Service failed")
        {
            Some(Event::Connect(ref mut sender)) => {
                println!("Connected to the server");
            }
            Some(Event::Disconnect(ref mut sender, ..)) => {
                println!("Disconnected from the server");
                break;
            }
            Some(Event::Receive {
                ref packet,
                ref mut sender,
                ..
            }) => {
                println!("Received a packet: {:?}", packet.data());
                hello();
            }
            _ => (),
        }
    }
}

fn hello() {
    println!("Hello, world!");
}
