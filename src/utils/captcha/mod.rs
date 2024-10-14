use serde::{Deserialize, Serialize};

pub mod capsolver;

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum CaptchaProvider {
    #[default]
    CapSolver,
}

pub fn solve_captcha(provider: CaptchaProvider, sitekey: &str) -> Option<String> {
    match provider {
        CaptchaProvider::CapSolver => capsolver::solve_captcha(sitekey),
    }
}