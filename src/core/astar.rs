use gtitem_r::structs::ItemDatabase;
use std::sync::RwLock;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Arc,
};

use super::Bot;

pub struct AStar {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<Node>,
    pub item_database: Arc<RwLock<ItemDatabase>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub g: u32,
    pub h: u32,
    pub f: u32,
    pub x: u32,
    pub y: u32,
    pub collision_type: u8,
}

impl Node {
    pub fn new(x: u32, y: u32, collision_type: u8) -> Node {
        Node {
            g: 0,
            h: 0,
            f: 0,
            x,
            y,
            collision_type,
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then_with(|| other.h.cmp(&self.h))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AStar {
    pub fn new(item_database: Arc<RwLock<ItemDatabase>>) -> AStar {
        AStar {
            width: 0,
            height: 0,
            grid: Vec::new(),
            item_database,
        }
    }

    pub fn reset(&mut self) {
        self.width = 0;
        self.height = 0;
        self.grid.clear();
    }

    pub fn update(&mut self, bot: &Bot) {
        self.reset();
        let world = bot.world.read().unwrap();
        self.width = world.width;
        self.height = world.height;
        for i in 0..world.tiles.len() {
            let x = (i as u32) % world.width;
            let y = (i as u32) / world.width;
            let item_database = self.item_database.read().unwrap();
            let item = item_database
                .get_item(&(world.tiles[i].foreground_item_id as u32))
                .unwrap();
            let collision_type = item.collision_type;
            self.grid.push(Node::new(x, y, collision_type));
        }
    }

    pub fn find_path(&self, from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> Option<Vec<Node>> {
        let mut open_list = BinaryHeap::new();
        let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
        let mut closed_set: HashSet<(u32, u32)> = HashSet::new();

        let start_index = (from_y * self.width + from_x) as usize;
        let start_node = self.grid[start_index].clone();
        let mut start_node = start_node;
        start_node.g = 0;
        start_node.h = self.calculate_h(from_x, from_y, to_x, to_y);
        start_node.f = start_node.g + start_node.h;
        open_list.push(start_node);

        while let Some(current_node) = open_list.pop() {
            if current_node.x == to_x && current_node.y == to_y {
                return Some(self.reconstruct_path(&came_from, (to_x, to_y), (from_x, from_y)));
            }

            if closed_set.contains(&(current_node.x, current_node.y)) {
                continue;
            }

            closed_set.insert((current_node.x, current_node.y));

            let neighbors = self.get_neighbors(&current_node);

            for neighbor in neighbors {
                if closed_set.contains(&(neighbor.x, neighbor.y)) {
                    continue;
                }

                let tentative_g = current_node.g + self.movement_cost(&current_node, &neighbor);

                let neighbor_index = (neighbor.y * self.width + neighbor.x) as usize;
                let mut neighbor_node = self.grid[neighbor_index].clone();
                if tentative_g < neighbor_node.g || neighbor_node.f == 0 {
                    neighbor_node.g = tentative_g;
                    neighbor_node.h = self.calculate_h(neighbor.x, neighbor.y, to_x, to_y);
                    neighbor_node.f = neighbor_node.g + neighbor_node.h;
                    open_list.push(neighbor_node);
                    came_from.insert((neighbor.x, neighbor.y), (current_node.x, current_node.y));
                }
            }
        }

        None
    }

    fn movement_cost(&self, from: &Node, to: &Node) -> u32 {
        let dx = if to.x > from.x {
            to.x - from.x
        } else {
            from.x - to.x
        };
        let dy = if to.y > from.y {
            to.y - from.y
        } else {
            from.y - to.y
        };
        if dx == 1 && dy == 1 {
            14
        } else {
            10
        }
    }

    fn calculate_h(&self, from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> u32 {
        let dx = if to_x > from_x {
            to_x - from_x
        } else {
            from_x - to_x
        };
        let dy = if to_y > from_y {
            to_y - from_y
        } else {
            from_y - to_y
        };
        14 * dx.min(dy) + 10 * (dx.max(dy) - dx.min(dy))
    }

    fn get_neighbors(&self, node: &Node) -> Vec<Node> {
        let mut neighbors = Vec::new();
        let directions = [
            (-1, 0),  // Left
            (1, 0),   // Right
            (0, -1),  // Up
            (0, 1),   // Down
            (-1, -1), // Up-Left
            (-1, 1),  // Down-Left
            (1, -1),  // Up-Right
            (1, 1),   // Down-Right
        ];

        for &(dx, dy) in &directions {
            let new_x = node.x as i32 + dx;
            let new_y = node.y as i32 + dy;

            if new_x >= 0 && new_x < self.width as i32 && new_y >= 0 && new_y < self.height as i32 {
                let index = (new_y as u32 * self.width + new_x as u32) as usize;
                let neighbor = &self.grid[index];

                if neighbor.collision_type == 1 || neighbor.collision_type == 6 {
                    continue;
                }

                if dx != 0 && dy != 0 {
                    let adj1_x = node.x as i32 + dx;
                    let adj1_y = node.y as i32;
                    let adj2_x = node.x as i32;
                    let adj2_y = node.y as i32 + dy;

                    if adj1_x < 0
                        || adj1_x >= self.width as i32
                        || adj2_y < 0
                        || adj2_y >= self.height as i32
                    {
                        continue;
                    }

                    let adj1_index = (adj1_y as u32 * self.width + adj1_x as u32) as usize;
                    let adj2_index = (adj2_y as u32 * self.width + adj2_x as u32) as usize;

                    let adj1 = &self.grid[adj1_index];
                    let adj2 = &self.grid[adj2_index];
                    if adj1.collision_type == 1 || adj2.collision_type == 1 {
                        continue;
                    }
                }

                neighbors.push(neighbor.clone());
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
            current = match came_from.get(&current) {
                Some(&prev) => prev,
                None => break,
            };
        }

        if let Some(start_node) = self
            .grid
            .iter()
            .find(|node| node.x == start.0 && node.y == start.1)
        {
            path.push(start_node.clone());
        }

        path.reverse();
        path
    }
}
