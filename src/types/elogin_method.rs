#[derive(Debug)]
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
