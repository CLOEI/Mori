#[repr(u32)]
#[derive(Debug)]
pub enum NetMessage {
    Unknown,
    ServerHello,
    GenericText,
    GameMessage,
    GamePacket,
    Error,
    Track,
    ClientLogRequest,
    ClientLogResponse,
    Max,
}

impl From<u32> for NetMessage {
    fn from(value: u32) -> Self {
        match value {
            0 => NetMessage::Unknown,
            1 => NetMessage::ServerHello,
            2 => NetMessage::GenericText,
            3 => NetMessage::GameMessage,
            4 => NetMessage::GamePacket,
            5 => NetMessage::Error,
            6 => NetMessage::Track,
            7 => NetMessage::ClientLogRequest,
            8 => NetMessage::ClientLogResponse,
            9 => NetMessage::Max,
            _ => NetMessage::Unknown,
        }
    }
}