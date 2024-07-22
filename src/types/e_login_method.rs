use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ELoginMethod {
    UBISOFT,
    APPLE,
    GOOGLE,
    LEGACY,
}
