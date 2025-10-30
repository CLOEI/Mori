use crate::types::bot::{Automation, DelayConfig};
use std::sync::Mutex;

#[derive(Debug)]
pub struct BotConfiguration {
    automation: Mutex<Automation>,
    delay_config: Mutex<DelayConfig>,
}

impl BotConfiguration {
    pub fn new() -> Self {
        Self {
            automation: Mutex::new(Automation::default()),
            delay_config: Mutex::new(DelayConfig::default()),
        }
    }

    // Automation getters/setters

    pub fn auto_collect(&self) -> bool {
        self.automation.lock().unwrap().auto_collect
    }

    pub fn set_auto_collect(&self, enabled: bool) {
        let mut auto = self.automation.lock().unwrap();
        auto.auto_collect = enabled;
    }

    pub fn auto_reconnect(&self) -> bool {
        self.automation.lock().unwrap().auto_reconnect
    }

    pub fn set_auto_reconnect(&self, enabled: bool) {
        let mut auto = self.automation.lock().unwrap();
        auto.auto_reconnect = enabled;
    }

    // Delay config getters/setters

    pub fn findpath_delay(&self) -> u32 {
        self.delay_config.lock().unwrap().findpath_delay
    }

    pub fn set_findpath_delay(&self, delay: u32) {
        let mut delays = self.delay_config.lock().unwrap();
        delays.findpath_delay = delay;
    }

    pub fn punch_delay(&self) -> u32 {
        self.delay_config.lock().unwrap().punch_delay
    }

    pub fn set_punch_delay(&self, delay: u32) {
        let mut delays = self.delay_config.lock().unwrap();
        delays.punch_delay = delay;
    }

    pub fn place_delay(&self) -> u32 {
        self.delay_config.lock().unwrap().place_delay
    }

    pub fn set_place_delay(&self, delay: u32) {
        let mut delays = self.delay_config.lock().unwrap();
        delays.place_delay = delay;
    }

    /// Get all config at once (for API endpoints)
    pub fn get_all(&self) -> (Automation, DelayConfig) {
        let auto = self.automation.lock().unwrap();
        let delays = self.delay_config.lock().unwrap();
        (*auto, *delays)
    }

    /// Set all config at once
    pub fn set_all(&self, automation: Automation, delays: DelayConfig) {
        let mut auto = self.automation.lock().unwrap();
        let mut delay = self.delay_config.lock().unwrap();
        *auto = automation;
        *delay = delays;
    }
}

impl Default for BotConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_collect() {
        let config = BotConfiguration::new();
        assert!(!config.auto_collect());

        config.set_auto_collect(true);
        assert!(config.auto_collect());
    }

    #[test]
    fn test_auto_reconnect() {
        let config = BotConfiguration::new();
        assert!(!config.auto_reconnect());

        config.set_auto_reconnect(true);
        assert!(config.auto_reconnect());
    }

    #[test]
    fn test_delays() {
        let config = BotConfiguration::new();

        config.set_findpath_delay(100);
        assert_eq!(config.findpath_delay(), 100);

        config.set_punch_delay(200);
        assert_eq!(config.punch_delay(), 200);

        config.set_place_delay(300);
        assert_eq!(config.place_delay(), 300);
    }

    #[test]
    fn test_get_all() {
        let config = BotConfiguration::new();
        config.set_auto_collect(true);
        config.set_findpath_delay(150);

        let (automation, delays) = config.get_all();
        assert!(automation.auto_collect);
        assert_eq!(delays.findpath_delay, 150);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let config = Arc::new(BotConfiguration::new());
        let mut handles = vec![];

        // Spawn 10 threads modifying config
        for i in 0..10 {
            let config = Arc::clone(&config);
            let handle = thread::spawn(move || {
                config.set_findpath_delay(i * 10);
                config.set_auto_collect(i % 2 == 0);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
