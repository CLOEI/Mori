#[repr(u32)]
#[derive(Debug)]
enum EPacketType {
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
