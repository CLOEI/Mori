use crate::TokenFetcher;
use crate::server::DashboardLinks;
use crate::types::bot::LoginVia;
use crate::types::login_info::LoginInfo;
use crate::types::server_data::ServerData;
use std::sync::{Mutex, MutexGuard};

pub struct AuthenticationContext {
    login_via: LoginVia,
    login_info: Mutex<Option<LoginInfo>>,
    server_data: Mutex<Option<ServerData>>,
    dashboard_links: Mutex<Option<DashboardLinks>>,
    token_fetcher: Option<TokenFetcher>,
}

impl AuthenticationContext {
    pub fn new(login_via: LoginVia, token_fetcher: Option<TokenFetcher>) -> Self {
        Self {
            login_via,
            login_info: Mutex::new(None),
            server_data: Mutex::new(None),
            dashboard_links: Mutex::new(None),
            token_fetcher,
        }
    }

    pub fn login_via(&self) -> LoginVia {
        self.login_via.clone()
    }

    pub fn set_login_via(&mut self, login_via: LoginVia) {
        self.login_via = login_via;
    }

    pub fn login_info(&self) -> MutexGuard<'_, Option<LoginInfo>> {
        self.login_info.lock().unwrap()
    }

    pub fn try_login_info(&self) -> Option<MutexGuard<'_, Option<LoginInfo>>> {
        self.login_info.try_lock().ok()
    }

    pub fn server_data(&self) -> MutexGuard<'_, Option<ServerData>> {
        self.server_data.lock().unwrap()
    }

    pub fn server_data_clone(&self) -> Option<ServerData> {
        self.server_data.lock().unwrap().clone()
    }

    pub fn dashboard_links(&self) -> MutexGuard<'_, Option<DashboardLinks>> {
        self.dashboard_links.lock().unwrap()
    }

    pub fn dashboard_links_clone(&self) -> Option<DashboardLinks> {
        self.dashboard_links.lock().unwrap().clone()
    }

    pub fn token_fetcher(&self) -> Option<&TokenFetcher> {
        self.token_fetcher.as_ref()
    }

    pub fn set_token_fetcher(&mut self, fetcher: Option<TokenFetcher>) {
        self.token_fetcher = fetcher;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_defaults() {
        let auth = AuthenticationContext::new(LoginVia::default(), None);
        assert!(auth.login_info().is_none());
        assert!(auth.server_data_clone().is_none());
        assert!(auth.dashboard_links_clone().is_none());
        assert!(auth.token_fetcher().is_none());
    }

    #[test]
    fn test_login_info_updates() {
        let auth = AuthenticationContext::new(LoginVia::default(), None);
        {
            let mut login_info = auth.login_info();
            *login_info = Some(LoginInfo::new());
        }

        assert!(auth.login_info().is_some());
    }

    #[test]
    fn test_access_from_multiple_threads() {
        let auth = Arc::new(AuthenticationContext::new(LoginVia::default(), None));

        let mut handles = Vec::new();
        for _ in 0..5 {
            let auth = Arc::clone(&auth);
            handles.push(thread::spawn(move || {
                let mut info = auth.login_info();
                *info = Some(LoginInfo::new());
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(auth.login_info().is_some());
    }
}
