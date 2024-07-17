use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

pub struct World {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tile_count: u32,
}

impl World {
    pub fn new() -> World {
        World {
            name: String::new(),
            width: 0,
            height: 0,
            tile_count: 0,
        }
    }

    pub fn parse(&mut self, data: &[u8]) {
        // first 6 byte is unknown
        let mut data = &data[6..];
        let str_len = data.read_u16::<LittleEndian>().unwrap();
        let mut name = vec![0; str_len as usize];
        data.read_exact(&mut name).unwrap();
        let width = data.read_u32::<LittleEndian>().unwrap();
        let height = data.read_u32::<LittleEndian>().unwrap();
        let tile_count = data.read_u32::<LittleEndian>().unwrap();

        self.name = String::from_utf8_lossy(&name).to_string();
        self.width = width;
        self.height = height;
        self.tile_count = tile_count;
    }
}
