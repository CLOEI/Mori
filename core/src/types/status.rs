#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ENetStatus {
    Connecting,
    Connected,
    Reconnecting,
    Disconnected,
}

impl std::fmt::Display for ENetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ENetStatus::Connecting => write!(f, "Connecting"),
            ENetStatus::Connected => write!(f, "Connected"),
            ENetStatus::Reconnecting => write!(f, "Reconnecting"),
            ENetStatus::Disconnected => write!(f, "Disconnected"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerStatus {
    FetchingServerData,
    ConnectingToServer,
    InGame,
}

impl std::fmt::Display for PeerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerStatus::FetchingServerData => write!(f, "Fetching Server Data"),
            PeerStatus::ConnectingToServer => write!(f, "Connecting to Server"),
            PeerStatus::InGame => write!(f, "In Game"),
        }
    }
}
