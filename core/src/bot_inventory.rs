use crate::inventory::{Inventory, InventoryItem};
use std::sync::{Mutex, atomic::{AtomicI32, Ordering}};

#[derive(Debug)]
pub struct BotInventory {
    items: Mutex<Inventory>,
    gems: AtomicI32,
}

impl BotInventory {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Inventory::new()),
            gems: AtomicI32::new(0),
        }
    }

    // Gems operations

    pub fn gems(&self) -> i32 {
        self.gems.load(Ordering::SeqCst)
    }

    pub fn set_gems(&self, gems: i32) {
        self.gems.store(gems, Ordering::SeqCst);
    }

    pub fn add_gems(&self, delta: i32) -> i32 {
        self.gems.fetch_add(delta, Ordering::SeqCst)
    }

    // Inventory operations

    pub fn parse(&self, data: &[u8]) {
        let mut inv = self.items.lock().unwrap();
        inv.parse(data);
    }

    pub fn get_item_count(&self, item_id: u16) -> u8 {
        let inv = self.items.lock().unwrap();
        inv.items.get(&item_id).map(|item| item.amount).unwrap_or(0)
    }

    pub fn has_item(&self, item_id: u16, count: u8) -> bool {
        self.get_item_count(item_id) >= count
    }

    pub fn size_and_count(&self) -> (u32, u16) {
        let inv = self.items.lock().unwrap();
        (inv.size, inv.item_count)
    }

    pub fn get_all_items(&self) -> Vec<(u16, InventoryItem)> {
        let inv = self.items.lock().unwrap();
        inv.items.iter().map(|(id, item)| (*id, item.clone())).collect()
    }

    pub fn update(&self, new_inventory: Inventory) {
        let mut inv = self.items.lock().unwrap();
        *inv = new_inventory;
    }

    pub fn add_item(&self, item_id: u16, amount: u8) {
        let mut inv = self.items.lock().unwrap();
        inv.items.entry(item_id)
            .and_modify(|item| item.amount = item.amount.saturating_add(amount))
            .or_insert(InventoryItem {
                id: item_id,
                amount,
                flag: 0,
            });
        inv.item_count = inv.items.len() as u16;
    }

    pub fn remove_item(&self, item_id: u16, amount: u8) -> bool {
        let mut inv = self.items.lock().unwrap();
        if let Some(item) = inv.items.get_mut(&item_id) {
            if item.amount >= amount {
                item.amount -= amount;
                if item.amount == 0 {
                    inv.items.remove(&item_id);
                    inv.item_count = inv.items.len() as u16;
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn try_get_snapshot(&self) -> Option<InventorySnapshot> {
        self.items.try_lock().ok().map(|inv| {
            let mut item_amounts = std::collections::HashMap::with_capacity(inv.items.len());
            for (&item_id, item) in &inv.items {
                item_amounts.insert(item_id, item.amount);
            }
            InventorySnapshot {
                size: inv.size,
                item_count: inv.items.len() as u32,
                item_amounts,
            }
        })
    }

    pub fn with_inventory_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Inventory) -> R,
    {
        let mut inv = self.items.lock().unwrap();
        f(&mut inv)
    }
}

impl Default for BotInventory {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InventorySnapshot {
    pub size: u32,
    pub item_count: u32,
    pub item_amounts: std::collections::HashMap<u16, u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gems() {
        let inv = BotInventory::new();
        assert_eq!(inv.gems(), 0);

        inv.set_gems(1000);
        assert_eq!(inv.gems(), 1000);

        inv.add_gems(500);
        assert_eq!(inv.gems(), 1500);

        inv.add_gems(-200);
        assert_eq!(inv.gems(), 1300);
    }

    #[test]
    fn test_item_management() {
        let inv = BotInventory::new();

        inv.add_item(100, 50);
        assert_eq!(inv.get_item_count(100), 50);

        inv.add_item(100, 30);
        assert_eq!(inv.get_item_count(100), 80);

        assert!(inv.remove_item(100, 30));
        assert_eq!(inv.get_item_count(100), 50);

        assert!(!inv.remove_item(100, 100)); // Not enough
        assert_eq!(inv.get_item_count(100), 50); // Unchanged
    }

    #[test]
    fn test_has_item() {
        let inv = BotInventory::new();
        inv.add_item(200, 75);

        assert!(inv.has_item(200, 50));
        assert!(inv.has_item(200, 75));
        assert!(!inv.has_item(200, 76));
        assert!(!inv.has_item(999, 1));
    }

    #[test]
    fn test_size_and_count() {
        let inv = BotInventory::new();
        inv.add_item(1, 10);
        inv.add_item(2, 20);

        let (size, count) = inv.size_and_count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_try_get_snapshot() {
        let inv = BotInventory::new();
        inv.add_item(10, 50);
        inv.add_item(20, 100);

        let snapshot = inv.try_get_snapshot().unwrap();
        assert_eq!(snapshot.item_count, 2);
        assert_eq!(snapshot.item_amounts.get(&10), Some(&50));
        assert_eq!(snapshot.item_amounts.get(&20), Some(&100));
    }

    #[test]
    fn test_concurrent_gems() {
        use std::sync::Arc;
        use std::thread;

        let inv = Arc::new(BotInventory::new());
        let mut handles = vec![];

        // Spawn 10 threads adding gems
        for i in 0..10 {
            let inv = Arc::clone(&inv);
            let handle = thread::spawn(move || {
                inv.add_gems(i * 10);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Sum should be 0+10+20+...+90 = 450
        assert_eq!(inv.gems(), 450);
    }

    #[test]
    fn test_concurrent_inventory() {
        use std::sync::Arc;
        use std::thread;

        let inv = Arc::new(BotInventory::new());
        let mut handles = vec![];

        // Spawn 5 threads adding different items
        for i in 0..5 {
            let inv = Arc::clone(&inv);
            let handle = thread::spawn(move || {
                inv.add_item(i as u16, 10);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should have 5 different items
        let items = inv.get_all_items();
        assert_eq!(items.len(), 5);
    }
}
