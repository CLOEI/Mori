use std::sync::{
    Mutex, RwLock, RwLockReadGuard,
    atomic::{AtomicU32, Ordering},
};

#[derive(Debug)]
pub struct RuntimeContext {
    net_id: Mutex<u32>,
    user_id: Mutex<u32>,
    ping: AtomicU32,
    logs: RwLock<Vec<String>>,
    is_running: Mutex<bool>,
    is_redirecting: Mutex<bool>,
}

impl RuntimeContext {
    pub fn new() -> Self {
        Self {
            net_id: Mutex::new(0),
            user_id: Mutex::new(0),
            ping: AtomicU32::new(0),
            logs: RwLock::new(Vec::new()),
            is_running: Mutex::new(true),
            is_redirecting: Mutex::new(false),
        }
    }

    pub fn net_id(&self) -> u32 {
        *self.net_id.lock().unwrap()
    }

    pub fn set_net_id(&self, value: u32) {
        let mut net_id = self.net_id.lock().unwrap();
        *net_id = value;
    }

    pub fn user_id(&self) -> u32 {
        *self.user_id.lock().unwrap()
    }

    pub fn set_user_id(&self, value: u32) {
        let mut user_id = self.user_id.lock().unwrap();
        *user_id = value;
    }

    pub fn ping(&self) -> u32 {
        self.ping.load(Ordering::Relaxed)
    }

    pub fn set_ping(&self, value: u32) {
        self.ping.store(value, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    pub fn set_running(&self, running: bool) {
        let mut state = self.is_running.lock().unwrap();
        *state = running;
    }

    pub fn is_redirecting(&self) -> bool {
        *self.is_redirecting.lock().unwrap()
    }

    pub fn set_redirecting(&self, redirecting: bool) {
        let mut state = self.is_redirecting.lock().unwrap();
        *state = redirecting;
    }

    pub fn push_log<S: Into<String>>(&self, message: S) {
        let mut logs = self.logs.write().unwrap();
        logs.push(message.into());
    }

    pub fn clear_logs(&self) {
        let mut logs = self.logs.write().unwrap();
        logs.clear();
    }

    pub fn logs(&self) -> RwLockReadGuard<'_, Vec<String>> {
        self.logs.read().unwrap()
    }

    pub fn logs_snapshot(&self) -> Vec<String> {
        self.logs.read().unwrap().clone()
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_defaults() {
        let runtime = RuntimeContext::new();
        assert_eq!(runtime.net_id(), 0);
        assert_eq!(runtime.ping(), 0);
        assert!(runtime.is_running());
        assert!(!runtime.is_redirecting());
        assert!(runtime.logs().is_empty());
    }

    #[test]
    fn test_setters() {
        let runtime = RuntimeContext::new();
        runtime.set_net_id(42);
        runtime.set_ping(123);
        runtime.set_running(false);
        runtime.set_redirecting(true);
        runtime.push_log("hello");

        assert_eq!(runtime.net_id(), 42);
        assert_eq!(runtime.ping(), 123);
        assert!(!runtime.is_running());
        assert!(runtime.is_redirecting());
        assert_eq!(runtime.logs_snapshot(), vec!["hello".to_string()]);
    }

    #[test]
    fn test_concurrent_updates() {
        let runtime = Arc::new(RuntimeContext::new());
        let mut handles = Vec::new();

        for i in 0..10 {
            let runtime = Arc::clone(&runtime);
            handles.push(thread::spawn(move || {
                runtime.set_net_id(i);
                runtime.set_ping(i as u32);
                runtime.push_log(format!("log {}", i));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(!runtime.logs().is_empty());
    }
}
