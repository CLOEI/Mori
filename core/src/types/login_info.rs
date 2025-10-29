use crate::utils;
use crate::utils::proton::HashMode;

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
    pub fcm_token: String,
}

impl LoginInfo {
    pub fn new() -> Self {
        let mut login_info = LoginInfo {
            tank_id_name: String::new(),
            tank_id_pass: String::new(),
            f: "1".to_string(),
            protocol: "217".to_string(),
            game_version: "5.33".to_string(),
            fz: "22243512".to_string(),
            cbits: "1024".to_string(),
            player_age: "20".to_string(),
            gdpr: "2".to_string(),
            fcm_token: String::new(),
            category: "_-5100".to_string(),
            total_play_time: "0".to_string(),
            klv: String::new(),
            hash2: String::new(),
            meta: String::new(),
            fhash: "-716928004".to_string(),
            rid: utils::random::hex(32, true),
            platform_id: "0,1,1".to_string(),
            device_version: "0".to_string(),
            country: "jp".to_string(),
            mac: utils::random::mac_address(),
            wk: utils::random::hex(32, true),
            zf: "31631978".to_string(),
            user: String::new(),
            uuid: String::new(),
            ltoken: "".to_string(),
            hash: "0".to_string(),
            requested_name: "".to_string(),
            lmode: "1".to_string(),
            token: String::new(),
            door_id: String::new(),
            aat: "0".to_string(),
        };

        login_info.klv = utils::proton::generate_klv(
            &login_info.protocol,
            &login_info.game_version,
            &login_info.rid,
        );
        login_info.hash = utils::proton::hash(
            &format!("{}RT", login_info.mac).as_bytes(),
            HashMode::NullTerminated,
        )
        .to_string();
        login_info.hash2 = utils::proton::hash(
            &format!("{}RT", utils::random::hex(16, true)).as_bytes(),
            HashMode::NullTerminated,
        )
        .to_string();

        login_info
    }

    pub fn to_string(&self) -> String {
        format!(
            "tankIDName|{}\ntankIDPass|{}\nrequestedName|{}\nf|{}\nprotocol|{}\ngame_version|{}\nfz|{}\ncbits|{}\nplayer_age|{}\nGDPR|{}\nFCMToken|{}\n\ncategory|{}\ntotalPlaytime|{}\nklv|{}\nhash2|{}\nmeta|{}\nfhash|{}\nrid|{}\nplatformID|{}\ndeviceVersion|{}\ncountry|{}\nhash|{}\nmac|{}\nwk|{}\nzf|{}lmode={}\n",
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
            self.fcm_token,
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
            self.zf,
            self.lmode,
        )
    }
}
