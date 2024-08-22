#[repr(u32)]
#[derive(Debug)]
pub enum EPacketType {
    NetMessageUnknown,
    NetMessageServerHello,
    NetMessageGenericText,
    NetMessageGameMessage,
    NetMessageGamePacket,
    NetMessageError,
    NetMessageTrack,
    NetMessageClientLogRequest,
    NetMessageClientLogResponse,
    NetMessageMax,
}

impl From<u32> for EPacketType {
    fn from(value: u32) -> Self {
        match value {
            0 => EPacketType::NetMessageUnknown,
            1 => EPacketType::NetMessageServerHello,
            2 => EPacketType::NetMessageGenericText,
            3 => EPacketType::NetMessageGameMessage,
            4 => EPacketType::NetMessageGamePacket,
            5 => EPacketType::NetMessageError,
            6 => EPacketType::NetMessageTrack,
            7 => EPacketType::NetMessageClientLogRequest,
            8 => EPacketType::NetMessageClientLogResponse,
            9 => EPacketType::NetMessageMax,
            _ => EPacketType::NetMessageUnknown,
        }
    }
}
