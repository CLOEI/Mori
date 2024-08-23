use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use gtitem_r::structs::ItemDatabase;

use super::Bot;

pub struct AStar {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<Node>,
    pub item_database: Arc<ItemDatabase>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub g: u32,
    pub h: u32,
    pub f: u32,
    pub x: u32,
    pub y: u32,
    pub collision_type: u8,
}

impl Node {
    pub fn new() -> Node {
        Node {
            g: 0,
            h: 0,
            f: 0,
            x: 0,
            y: 0,
            collision_type: 0,
        }
    }
}

impl AStar {
    pub fn new(item_database: Arc<ItemDatabase>) -> AStar {
        AStar {
            width: 0,
            height: 0,
            grid: Vec::new(),
            item_database,
        }
    }

    pub fn update(&mut self, bot: &Arc<Bot>) {
        let mut world = bot.world.lock().unwrap();
        self.width = world.width;
        self.height = world.height;
        for i in 0..world.tiles.len() {
            let mut node = Node::new();
            node.x = (i as u32) % world.width;
            node.y = (i as u32) / world.width;
            let item = self
                .item_database
                .get_item(&(world.tiles[i].foreground_item_id as u32))
                .unwrap();
            node.collision_type = item.collision_type;
            self.grid.push(node);
        }
    }

    pub fn find_path(&self, from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> Option<Vec<Node>> {
        let mut open_list: Vec<Node> = Vec::new();
        let mut closed_list: Vec<Node> = Vec::new();
        let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();

        let start_index = (from_y * self.width + from_x) as usize;
        let mut start_node = self.grid[start_index].clone();
        start_node.g = 0;
        start_node.h = self.calculate_h(from_x, from_y, to_x, to_y);
        start_node.f = start_node.g + start_node.h;
        open_list.push(start_node);

        while !open_list.is_empty() {
            let current_index = open_list
                .iter()
                .enumerate()
                .min_by_key(|&(_, node)| node.f)
                .unwrap()
                .0;
            let current_node = open_list.remove(current_index);

            if current_node.x == to_x && current_node.y == to_y {
                return Some(self.reconstruct_path(&came_from, (to_x, to_y), (from_x, from_y)));
            }

            let children = self.get_neighbors(&current_node);

            for mut child in children {
                if closed_list
                    .iter()
                    .any(|closed_node| closed_node.x == child.x && closed_node.y == child.y)
                {
                    continue;
                }

                child.g = current_node.g + 1;
                child.h = self.calculate_h(child.x, child.y, to_x, to_y);
                child.f = child.g + child.h;

                if let Some(open_node) = open_list
                    .iter()
                    .find(|node| node.x == child.x && node.y == child.y)
                {
                    if child.g > open_node.g {
                        continue;
                    }
                }

                came_from.insert((child.x, child.y), (current_node.x, current_node.y));
                open_list.push(child);
            }

            closed_list.push(current_node);
        }

        None
    }

    fn calculate_h(&self, from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> u32 {
        let dx = (from_x as i32 - to_x as i32).abs();
        let dy = (from_y as i32 - to_y as i32).abs();
        if dx == dy {
            14 * dx as u32
        } else {
            10 * (dx + dy) as u32
        }
    }

    fn get_neighbors(&self, node: &Node) -> Vec<Node> {
        let mut neighbors = Vec::new();
        let directions = [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ];

        for (dx, dy) in directions.iter() {
            let new_x = node.x as i32 + dx;
            let new_y = node.y as i32 + dy;

            if new_x >= 0 && new_x < self.width as i32 && new_y >= 0 && new_y < self.height as i32 {
                let index = (new_y as u32 * self.width + new_x as u32) as usize;
                let neighbor = &self.grid[index];

                if neighbor.collision_type != 1 {
                    neighbors.push(neighbor.clone());
                }
            }
        }

        neighbors
    }

    fn reconstruct_path(
        &self,
        came_from: &HashMap<(u32, u32), (u32, u32)>,
        current: (u32, u32),
        start: (u32, u32),
    ) -> Vec<Node> {
        let mut path = Vec::new();
        let mut current = current;
        while current != start {
            if let Some(node) = self
                .grid
                .iter()
                .find(|node| node.x == current.0 && node.y == current.1)
            {
                path.push(node.clone());
            }
            current = *came_from.get(&current).unwrap();
        }
        path.push(
            self.grid
                .iter()
                .find(|node| node.x == start.0 && node.y == start.1)
                .unwrap()
                .clone(),
        );
        path.reverse();
        path
    }
}
