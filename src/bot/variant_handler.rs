use enet::Peer;
use spdlog::info;

use crate::types::tank_packet_type::TankPacketType;
use crate::utils::variant::VariantList;

use super::Bot;

pub fn handle(bot: &Bot, peer: &Peer<()>, pkt: &TankPacketType, data: &[u8]) {
    let variant = VariantList::deserialize(&data).unwrap();
    let function = variant.get(0).unwrap().as_string();
    info!("Received function call: {}", function);
}
