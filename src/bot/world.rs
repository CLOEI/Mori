use std::io::{Cursor, Read};
use std::{fs::File, io::Write, sync::Arc};

use byteorder::{LittleEndian, ReadBytesExt};
use gtitem_r::structs::ItemDatabase;
use spdlog::error;

pub struct World {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tile_count: u32,
    pub tiles: Vec<Tile>,
    pub base_weather: u16,
    pub current_weather: u16,
    pub item_database: Arc<ItemDatabase>,
}

#[derive(Debug)]
pub struct Tile {
    pub foreground_item_id: u16,
    pub background_item_id: u16,
    pub first_flag: u8,
    pub second_flag: u8,
    pub tile_type: TileType,
    pub dropped: Dropped,
}

#[derive(Debug)]
pub enum TileType {
    Basic,
    Door {
        text: String,
        unknown_1: u8,
    },
    Sign {
        text: String,
        unknown_1: u32,
    },
    Lock {
        settings: u8,
        owner_uid: u32,
        access_count: u32,
        access_uids: Vec<u32>,
    },
    Seed {
        time_passed: u32,
        item_on_tree: u8,
    },
    Mailbox {
        unknown_1: String,
        unknown_2: String,
        unknown_3: String,
        unknown_4: u8,
    },
    Bulletin {
        unknown_1: String,
        unknown_2: String,
        unknown_3: String,
        unknown_4: u8,
    },
    Dice {
        symbol: u8,
    },
    ChemicalSource {
        time_passed: u32,
    },
    AchievemntBlock {
        unknown_1: u32,
        tile_type: u8,
    },
    HearthMonitor {
        unknown_1: u32,
        player_name: String,
    },
    DonationBox {
        unknown_1: String,
        unknown_2: String,
        unknown_3: String,
        unknown_4: u8,
    },
    Mannequin {
        text: String,
        unknown_1: u8,
        clothing_1: u32,
        clothing_2: u16,
        clothing_3: u16,
        clothing_4: u16,
        clothing_5: u16,
        clothing_6: u16,
        clothing_7: u16,
        clothing_8: u16,
        clothing_9: u16,
        clothing_10: u16,
    },
    BunnyEgg {
        egg_placed: u32,
    },
    GamePack {
        team: u8,
    },
    GameGenerator {},
    XenoniteCrystal {
        unknown_1: u8,
        unknown_2: u32,
    },
    PhoneBooth {
        clothing_1: u16,
        clothing_2: u16,
        clothing_3: u16,
        clothing_4: u16,
        clothing_5: u16,
        clothing_6: u16,
        clothing_7: u16,
        clothing_8: u16,
        clothing_9: u16,
    },
    Crystal {
        unknown_1: String,
    },
    CrimeInProgress {
        unknown_1: String,
        unknown_2: u32,
        unknown_3: u8,
    },
    DisplayBlock {
        item_id: u32,
    },
    VendingMachine {
        item_id: u32,
        price: i32,
    },
    GivingTree {
        unknown_1: u16,
        unknown_2: u32,
    },
    CountryFlag {
        country: String,
    },
    WeatherMachine {
        item_id: u32,
    },
}

#[derive(Debug)]
pub struct Dropped {
    pub items_count: u32,
    pub last_dropped_item_uid: u32,
    pub items: Vec<DroppedItem>,
}

#[derive(Debug)]
pub struct DroppedItem {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub count: u8,
    pub flags: u8,
    pub uid: u32,
}

impl Tile {
    pub fn new(
        foreground_item_id: u16,
        background_item_id: u16,
        first_flag: u8,
        second_flag: u8,
    ) -> Tile {
        Tile {
            foreground_item_id,
            background_item_id,
            first_flag,
            second_flag,
            tile_type: TileType::Basic,
            dropped: Dropped {
                items_count: 0,
                last_dropped_item_uid: 0,
                items: Vec::new(),
            },
        }
    }
}

impl World {
    pub fn new(item_database: Arc<ItemDatabase>) -> World {
        World {
            name: String::new(),
            width: 0,
            height: 0,
            tile_count: 0,
            tiles: Vec::new(),
            base_weather: 0,
            current_weather: 0,
            item_database,
        }
    }

    pub fn parse(&mut self, data: &[u8]) {
        // first 6 byte is unknown
        let mut data = Cursor::new(&data[6..]);
        // save data to file
        let mut file = File::create("world.dat").unwrap();
        file.write_all(data.get_ref()).unwrap();

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

        // tiles
        for _ in 0..tile_count {
            let mut tile = Tile::new(0, 0, 0, 0);
            tile.foreground_item_id = data.read_u16::<LittleEndian>().unwrap();
            tile.background_item_id = data.read_u16::<LittleEndian>().unwrap();
            data.read_u16::<LittleEndian>().unwrap(); // unknown
            tile.first_flag = data.read_u8().unwrap();
            tile.second_flag = data.read_u8().unwrap();
            let foreground_item = self
                .item_database
                .get_item(&(tile.foreground_item_id as u32))
                .unwrap();
            let background_item = self
                .item_database
                .get_item(&(tile.background_item_id as u32))
                .unwrap();
            let item_type = if tile.foreground_item_id == 0 && tile.background_item_id != 0 {
                self.action_to_item_type(background_item.action_type as u32)
            } else {
                self.action_to_item_type(foreground_item.action_type as u32)
            };

            self.get_extra_tile_data(&mut tile, &mut data, item_type);
            tile.dropped.items_count = data.read_u32::<LittleEndian>().unwrap();
            tile.dropped.last_dropped_item_uid = data.read_u32::<LittleEndian>().unwrap();
            for _ in 0..tile.dropped.items_count {
                let id = data.read_u32::<LittleEndian>().unwrap();
                let x = data.read_f32::<LittleEndian>().unwrap();
                let y = match data.read_f32::<LittleEndian>() {
                    Ok(y) => y,
                    Err(_) => {
                        println!("idk");
                        0.0
                    }
                };
                let count = data.read_u8().unwrap();
                let flags = data.read_u8().unwrap();
                let uid = data.read_u32::<LittleEndian>().unwrap();
                tile.dropped.items.push(DroppedItem {
                    id,
                    x,
                    y,
                    count,
                    flags,
                    uid,
                });
            }
            let pos = data.position();
            println!("pos: {}", pos);
            if pos == 1444 {
                println!("breakpoint for testing");
            }
            self.tiles.push(tile);
        }

        self.base_weather = data.read_u16::<LittleEndian>().unwrap();
        data.read_u16::<LittleEndian>().unwrap(); // unknown
        self.current_weather = data.read_u16::<LittleEndian>().unwrap();

        error!("{:?}", self.tiles);
    }

    fn get_extra_tile_data(&self, tile: &mut Tile, data: &mut Cursor<&[u8]>, item_type: u32) {
        match item_type {
            1 => {
                let str_len = data.read_u16::<LittleEndian>().unwrap();
                let mut text = vec![0; str_len as usize];
                data.read_exact(&mut text).unwrap();
                let text = String::from_utf8_lossy(&text).to_string();
                let unknown_1 = data.read_u8().unwrap();

                tile.tile_type = TileType::Door { text, unknown_1 };
            }
            2 => {
                let str_len = data.read_u16::<LittleEndian>().unwrap();
                let mut text = vec![0; str_len as usize];
                data.read_exact(&mut text).unwrap();
                let text = String::from_utf8_lossy(&text).to_string();
                let unknown_1 = data.read_u32::<LittleEndian>().unwrap();

                tile.tile_type = TileType::Sign { text, unknown_1 };
            }
            3 => {
                let settings = data.read_u8().unwrap();
                let owner_uid = data.read_u32::<LittleEndian>().unwrap();
                let access_count = data.read_u32::<LittleEndian>().unwrap();
                let mut access_uids = Vec::new();
                for _ in 0..access_count {
                    access_uids.push(data.read_u32::<LittleEndian>().unwrap());
                }

                tile.tile_type = TileType::Lock {
                    settings,
                    owner_uid,
                    access_count,
                    access_uids,
                };
            }
            4 => {
                let time_passed = data.read_u32::<LittleEndian>().unwrap();
                let item_on_tree = data.read_u8().unwrap();

                tile.tile_type = TileType::Seed {
                    time_passed,
                    item_on_tree,
                };
            }
            6 => {
                let unknown_1_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_1 = vec![0; unknown_1_len as usize];
                data.read_exact(&mut unknown_1).unwrap();

                let unknown_2_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_2 = vec![0; unknown_2_len as usize];
                data.read_exact(&mut unknown_2).unwrap();

                let unknown_3_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_3 = vec![0; unknown_3_len as usize];
                data.read_exact(&mut unknown_3).unwrap();

                let unknown_4 = data.read_u8().unwrap();

                tile.tile_type = TileType::Mailbox {
                    unknown_1: String::from_utf8_lossy(&unknown_1).to_string(),
                    unknown_2: String::from_utf8_lossy(&unknown_2).to_string(),
                    unknown_3: String::from_utf8_lossy(&unknown_3).to_string(),
                    unknown_4,
                };
            }
            7 => {
                let unknown_1_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_1 = vec![0; unknown_1_len as usize];
                data.read_exact(&mut unknown_1).unwrap();

                let unknown_2_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_2 = vec![0; unknown_2_len as usize];
                data.read_exact(&mut unknown_2).unwrap();

                let unknown_3_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_3 = vec![0; unknown_3_len as usize];
                data.read_exact(&mut unknown_3).unwrap();

                let unknown_4 = data.read_u8().unwrap();

                tile.tile_type = TileType::Bulletin {
                    unknown_1: String::from_utf8_lossy(&unknown_1).to_string(),
                    unknown_2: String::from_utf8_lossy(&unknown_2).to_string(),
                    unknown_3: String::from_utf8_lossy(&unknown_3).to_string(),
                    unknown_4,
                };
            }
            8 => {
                let symbol = data.read_u8().unwrap();

                tile.tile_type = TileType::Dice { symbol };
            }
            9 => {
                let time_passed = data.read_u32::<LittleEndian>().unwrap();

                tile.tile_type = TileType::ChemicalSource { time_passed };
            }
            10 => {
                let unknown_1 = data.read_u32::<LittleEndian>().unwrap();
                let tile_type = data.read_u8().unwrap();

                tile.tile_type = TileType::AchievemntBlock {
                    unknown_1,
                    tile_type,
                };
            }
            11 => {
                let unknown_1 = data.read_u32::<LittleEndian>().unwrap();
                let str_len = data.read_u16::<LittleEndian>().unwrap();
                let mut player_name = vec![0; str_len as usize];
                data.read_exact(&mut player_name).unwrap();
                let player_name = String::from_utf8_lossy(&player_name).to_string();

                tile.tile_type = TileType::HearthMonitor {
                    unknown_1,
                    player_name,
                };
            }
            12 => {
                let unknown_1_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_1 = vec![0; unknown_1_len as usize];
                data.read_exact(&mut unknown_1).unwrap();

                let unknown_2_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_2 = vec![0; unknown_2_len as usize];
                data.read_exact(&mut unknown_2).unwrap();

                let unknown_3_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_3 = vec![0; unknown_3_len as usize];
                data.read_exact(&mut unknown_3).unwrap();

                let unknown_4 = data.read_u8().unwrap();

                tile.tile_type = TileType::DonationBox {
                    unknown_1: String::from_utf8_lossy(&unknown_1).to_string(),
                    unknown_2: String::from_utf8_lossy(&unknown_2).to_string(),
                    unknown_3: String::from_utf8_lossy(&unknown_3).to_string(),
                    unknown_4,
                };
            }
            14 => {
                let str_len = data.read_u16::<LittleEndian>().unwrap();
                let mut text = vec![0; str_len as usize];
                data.read_exact(&mut text).unwrap();
                let text = String::from_utf8_lossy(&text).to_string();
                let unknown_1 = data.read_u8().unwrap();
                let clothing_1 = data.read_u32::<LittleEndian>().unwrap();
                let clothing_2 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_3 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_4 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_5 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_6 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_7 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_8 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_9 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_10 = data.read_u16::<LittleEndian>().unwrap();

                tile.tile_type = TileType::Mannequin {
                    text,
                    unknown_1,
                    clothing_1,
                    clothing_2,
                    clothing_3,
                    clothing_4,
                    clothing_5,
                    clothing_6,
                    clothing_7,
                    clothing_8,
                    clothing_9,
                    clothing_10,
                };
            }
            15 => {
                let egg_placed = data.read_u32::<LittleEndian>().unwrap();

                tile.tile_type = TileType::BunnyEgg { egg_placed };
            }
            16 => {
                let team = data.read_u8().unwrap();

                tile.tile_type = TileType::GamePack { team };
            }
            17 => {
                tile.tile_type = TileType::GameGenerator {};
            }
            18 => {
                let unknown_1 = data.read_u8().unwrap();
                let unknown_2 = data.read_u32::<LittleEndian>().unwrap();

                tile.tile_type = TileType::XenoniteCrystal {
                    unknown_1,
                    unknown_2,
                };
            }
            19 => {
                let clothing_1 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_2 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_3 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_4 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_5 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_6 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_7 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_8 = data.read_u16::<LittleEndian>().unwrap();
                let clothing_9 = data.read_u16::<LittleEndian>().unwrap();

                tile.tile_type = TileType::PhoneBooth {
                    clothing_1,
                    clothing_2,
                    clothing_3,
                    clothing_4,
                    clothing_5,
                    clothing_6,
                    clothing_7,
                    clothing_8,
                    clothing_9,
                };
            }
            20 => {
                let str_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_1 = vec![0; str_len as usize];
                data.read_exact(&mut unknown_1).unwrap();
                let unknown_1 = String::from_utf8_lossy(&unknown_1).to_string();

                tile.tile_type = TileType::Crystal { unknown_1 };
            }
            21 => {
                let str_len = data.read_u16::<LittleEndian>().unwrap();
                let mut unknown_1 = vec![0; str_len as usize];
                data.read_exact(&mut unknown_1).unwrap();
                let unknown_1 = String::from_utf8_lossy(&unknown_1).to_string();
                let unknown_2 = data.read_u32::<LittleEndian>().unwrap();
                let unknown_3 = data.read_u8().unwrap();

                tile.tile_type = TileType::CrimeInProgress {
                    unknown_1,
                    unknown_2,
                    unknown_3,
                };
            }
            23 => {
                let item_id = data.read_u32::<LittleEndian>().unwrap();

                tile.tile_type = TileType::DisplayBlock { item_id };
            }
            24 => {
                let item_id = data.read_u32::<LittleEndian>().unwrap();
                let price = data.read_i32::<LittleEndian>().unwrap();

                tile.tile_type = TileType::VendingMachine { item_id, price };
            }
            28 => {
                let unknown_1 = data.read_u16::<LittleEndian>().unwrap();
                let unknown_2 = data.read_u32::<LittleEndian>().unwrap();

                tile.tile_type = TileType::GivingTree {
                    unknown_1,
                    unknown_2,
                };
            }
            33 => {
                let country_len = data.read_u16::<LittleEndian>().unwrap();
                let mut country = vec![0; country_len as usize];
                data.read_exact(&mut country).unwrap();
                let country = String::from_utf8_lossy(&country).to_string();

                tile.tile_type = TileType::CountryFlag { country };
            }
            40 => {
                let item_id = data.read_u32::<LittleEndian>().unwrap();

                tile.tile_type = TileType::WeatherMachine { item_id };
            }
            _ => {
                tile.tile_type = TileType::Basic;
            }
        };
    }

    fn action_to_item_type(&self, input: u32) -> u32 {
        match input {
            2 => 1,
            3 => 3,
            10 => 2,
            13 => 1,
            19 => 4,
            26 => 1,
            33 => 6,
            34 => 7,
            36 => 8,
            38 => 9,
            40 => 10,
            43 => 1,
            46 => 11,
            47 => 12,
            48 => 13,
            49 => 14,
            51 => 15,
            52 => 16,
            53 => 17,
            54 => 18,
            55 => 19,
            56 => 20,
            57 => 21,
            59 => 22,
            61 => 23,
            62 => 24,
            63 => 25,
            65 => 26,
            66 => 27,
            67 => 28,
            68 => 29,
            71 => 30,
            72 => 31,
            73 => 32,
            74 => 33,
            75 => 34,
            76 => 35,
            77 => 36,
            78 => 37,
            79 => 38,
            80 => 39,
            81 => 40,
            82 => 41,
            83 => 43,
            84 => 44,
            85 => 45,
            86 => 33,
            87 => 47,
            88 => 48,
            89 => 49,
            92 => 51,
            _ => 0,
        }
    }
}
