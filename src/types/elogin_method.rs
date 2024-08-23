use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ELoginMethod {
    UBISOFT,
    APPLE,
    GOOGLE,
    LEGACY,
}

impl Default for ELoginMethod {
    fn default() -> Self {
        ELoginMethod::LEGACY
    }
}
