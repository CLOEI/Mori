use serde::{Deserialize, Serialize};

pub mod capsolver;
pub mod twocaptcha;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub enum CaptchaProvider {
    #[default]
    CapSolver,
    TwoCaptcha,
}

pub fn solve_captcha(provider: CaptchaProvider, sitekey: &str) -> Option<String> {
    match provider {
        CaptchaProvider::CapSolver => capsolver::solve_captcha(sitekey),
        CaptchaProvider::TwoCaptcha => twocaptcha::solve_captcha(sitekey),
    }
}
