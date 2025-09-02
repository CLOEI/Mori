use crate::utils;

#[derive(Debug, Default)]
pub struct LoginInfo {
    pub uuid: String,
    pub tank_id_name: String,
    pub tank_id_pass: String,
    pub protocol: String,
    pub fhash: String,
    pub mac: String,
    pub requested_name: String,
    pub hash2: String,
    pub fz: String,
    pub f: String,
    pub player_age: String,
    pub game_version: String,
    pub lmode: String,
    pub cbits: String,
    pub rid: String,
    pub gdpr: String,
    pub hash: String,
    pub category: String,
    pub token: String,
    pub total_play_time: String,
    pub door_id: String,
    pub klv: String,
    pub meta: String,
    pub platform_id: String,
    pub device_version: String,
    pub zf: String,
    pub country: String,
    pub user: String,
    pub wk: String,
    pub aat: String,
    pub ltoken: String,
}

impl LoginInfo {
    pub fn new() -> Self {
        let mut login_info = LoginInfo {
            uuid: String::new(),
            tank_id_name: String::new(),
            tank_id_pass: String::new(),
            protocol: "216".to_string(),
            fhash: "-716928004".to_string(),
            mac: utils::random::mac_address(),
            requested_name: "".to_string(),
            hash2: String::new(),
            fz: "20743704".to_string(),
            f: "1".to_string(),
            player_age: "20".to_string(),
            game_version: "5.26".to_string(),
            lmode: "1".to_string(),
            cbits: "1040".to_string(),
            rid: utils::random::hex(32, true),
            gdpr: "2".to_string(),
            hash: "0".to_string(),
            category: "_-5100".to_string(),
            token: String::new(),
            total_play_time: "0".to_string(),
            door_id: String::new(),
            klv: String::new(),
            meta: String::new(),
            platform_id: "0,1,1".to_string(),
            device_version: "0".to_string(),
            zf: "31631978".to_string(),
            country: "jp".to_string(),
            user: String::new(),
            wk: utils::random::hex(32, true),
            aat: "0".to_string(),
            ltoken: "" .to_string(),
        };

        login_info.klv = utils::proton::generate_klv(&login_info.protocol, &login_info.game_version, &login_info.rid);
        login_info.hash = utils::proton::hash_string(&format!("{}RT", login_info.mac)).to_string();
        login_info.hash2 = utils::proton::hash_string(&format!("{}RT", utils::random::hex(16, true))).to_string();
        
        login_info
    }

    pub fn to_string(&self) -> String {
        format!(
            "tankIDName|{}\ntankIDPass|{}\nrequestedName|{}\nf|{}\nprotocol|{}\ngame_version|{}\nfz|{}\ncbits|{}\nplayer_age|{}\nGDPR|{}\ncategory|{}\ntotalPlaytime|{}\nklv|{}\nhash2|{}\nmeta|{}\nfhash|{}\nrid|{}\nplatformID|{}\ndeviceVersion|{}\ncountry|{}\nhash|{}\nmac|{}\nwk|{}\nzf|{}\n",
            self.tank_id_name,
            self.tank_id_pass,
            self.requested_name,
            self.f,
            self.protocol,
            self.game_version,
            self.fz,
            self.cbits,
            self.player_age,
            self.gdpr,
            self.category,
            self.total_play_time,
            self.klv,
            self.hash2,
            self.meta,
            self.fhash,
            self.rid,
            self.platform_id,
            self.device_version,
            self.country,
            self.hash,
            self.mac,
            self.wk,
            self.zf
        )
    }
}