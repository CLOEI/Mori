use std::collections::HashMap;
use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Default, Clone)]
pub struct Inventory {
    pub size: u32,
    pub item_count: u16,
    pub items: HashMap<u16, InventoryItem>,
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub id: u16,
    pub amount: u8,
    pub flag: u8,
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            size: 0,
            item_count: 0,
            items: HashMap::new(),
        }
    }

    pub fn parse(&mut self, data: &[u8]) {
        self.reset();
        let mut data = Cursor::new(data);
        data.set_position(data.position() + 1);
        self.size = data.read_u32::<LittleEndian>().unwrap();
        self.item_count = data.read_u16::<LittleEndian>().unwrap();
        for _ in 0..self.item_count {
            let id = data.read_u16::<LittleEndian>().unwrap();
            let amount = data.read_u8().unwrap();
            let flag = data.read_u8().unwrap();
            self.items.insert(id, InventoryItem { id, amount, flag });
        }
    }

    pub fn reset(&mut self) {
        self.size = 0;
        self.item_count = 0;
        self.items.clear();
    }
}
