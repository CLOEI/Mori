use crate::Bot;
use crate::types::net_message::NetMessage;
use crate::utils::variant::VariantList;

pub fn handle(bot: &Bot, data: &[u8]) {
    let variant = VariantList::deserialize(&data).expect("Failed to deserialize variant list");
    let function_call: String = variant.get(0).unwrap().as_string();

    match function_call.as_str() {
        "OnSendToServer" => {
            let port = variant.get(1).unwrap().as_int32();
            let token = variant.get(2).unwrap().as_int32();
            let user_id = variant.get(3).unwrap().as_int32();
            let server_data = variant.get(4).unwrap().as_string();
            let parsed_server_data: Vec<String> = server_data.split('|').map(|s| s.trim_end().to_string()).collect();
            let aat = variant.get(5).unwrap().as_int32();

            let mut server_data_lock = bot.info.server_data.lock().unwrap();
            let server_data = server_data_lock.as_mut().unwrap();

            server_data.server = parsed_server_data[0].clone();
            server_data.port = port as u16;

            let mut is_redirecting_lock = bot.state.is_redirecting.lock().unwrap();
            *is_redirecting_lock = true;

            let mut login_info_lock = bot.info.login_info.lock().unwrap();
            let login_info = login_info_lock.as_mut().unwrap();

            login_info.token = token.to_string();
            login_info.user = user_id.to_string();
            login_info.door_id = parsed_server_data[1].clone();
            login_info.uuid = parsed_server_data[2].clone();
            login_info.aat = aat.to_string();

            bot.disconnect()
        }
        "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
            bot.send_packet(NetMessage::GenericText, "action|enter_game\n".to_string());
            let mut is_redirecting_lock = bot.state.is_redirecting.lock().unwrap();
            *is_redirecting_lock = false;
        }
        _ => {}
    }
}