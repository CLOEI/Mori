use paris::info;
use std::sync::Arc;

use crate::bot::{disconnect, send_packet};
use crate::types::epacket_type::EPacketType;
use crate::types::tank_packet::TankPacket;
use crate::utils::textparse;
use crate::utils::variant::VariantList;

use super::Bot;

pub fn handle(bot: &Arc<Bot>, pkt: &TankPacket, data: &[u8]) {
    let variant = VariantList::deserialize(&data).unwrap();
    let function_call: String = variant.get(0).unwrap().as_string();
    info!("Received function call: {}", function_call);

    match function_call.as_str() {
        "OnSendToServer" => {
            let port = variant.get(1).unwrap().as_int32();
            let token = variant.get(2).unwrap().as_int32();
            let user_id = variant.get(3).unwrap().as_int32();
            let server_data = variant.get(4).unwrap().as_string();
            let parsed_server_data = textparse::parse_and_store_as_vec(&server_data);

            let mut state = bot.state.lock().unwrap();
            let mut server = bot.server.lock().unwrap();
            let mut info = bot.info.lock().unwrap();

            state.is_redirecting = true;
            server.ip = parsed_server_data.get(0).unwrap().to_string();
            server.port = port as u16;
            info.login_info.token = token.to_string();
            info.login_info.user = user_id.to_string();
            info.login_info.door_id = parsed_server_data.get(1).unwrap().to_string();
            info.login_info.uuid = parsed_server_data.get(2).unwrap().to_string();
            disconnect(bot);
        }
        "OnSuperMainStartAcceptLogonHrdxs47254722215a" => {
            send_packet(
                bot,
                EPacketType::NetMessageGenericText,
                "action|enter_game\n".to_string(),
            );
            bot.state.lock().unwrap().is_redirecting = false;
        }
        "OnCountryState" => {}
        "OnDialogRequest" => {}
        "OnSetBux" => {}
        "OnConsoleMessage" => {
            let message = variant.get(1).unwrap().as_string();
            info!("Received console message: {}", message);
        }
        "OnSetPos" => {}
        "ShowStartFTUEPopup" => {}
        "OnFtueButtonDataSet" => {}
        "OnSpawn" => {}
        "OnTalkBubble" => {}
        "OnClearTutorialArrow" => {}
        _ => {}
    }
}
