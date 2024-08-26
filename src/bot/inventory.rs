use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Default, Clone)]
pub struct Inventory {
    pub size: u32,
    pub item_count: u16,
    pub items: Vec<InventoryItem>,
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub id: u16,
    pub amount: u16,
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            size: 0,
            item_count: 0,
            items: Vec::new(),
        }
    }

    pub fn parse(&mut self, data: &[u8]) {
        let mut data = Cursor::new(data);
        data.set_position(data.position() + 1);
        self.size = data.read_u32::<LittleEndian>().unwrap();
        self.item_count = data.read_u16::<LittleEndian>().unwrap();
        for _ in 0..self.item_count {
            let id = data.read_u16::<LittleEndian>().unwrap();
            let amount = data.read_u16::<LittleEndian>().unwrap();
            self.items.push(InventoryItem { id, amount });
        }
    }
}
