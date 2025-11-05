use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
};

pub struct AStar {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<u8>, // Store only collision types, not full nodes
    pub path_cache: HashMap<(u32, u32, u32, u32, bool), Option<Vec<(u32, u32)>>>, // Cache for paths (includes has_access)
    pub cache_hits: u32,
    pub cache_misses: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathNode {
    pub f: u32,
    pub g: u32,
    pub h: u32,
    pub x: u32,
    pub y: u32,
}

// Lightweight representation for the final path
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub g: u32,
    pub h: u32,
    pub f: u32,
    pub x: u32,
    pub y: u32,
    pub collision_type: u8,
}

impl PathNode {
    pub fn new(x: u32, y: u32, g: u32, h: u32) -> Self {
        Self {
            f: g + h,
            g,
            h,
            x,
            y,
        }
    }
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

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then_with(|| other.h.cmp(&self.h))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    pub fn new() -> AStar {
        AStar {
            width: 0,
            height: 0,
            grid: Vec::new(),
            path_cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn reset(&mut self) {
        self.width = 0;
        self.height = 0;
        self.grid.clear();
        if self.path_cache.len() > 1000 {
            self.path_cache.clear();
        }
    }

    pub fn update_from_collision_data(&mut self, width: u32, height: u32, collision_data: &[u8]) {
        if self.width != width || self.height != height {
            self.reset();
            self.width = width;
            self.height = height;
            self.grid.reserve_exact(collision_data.len());
        } else {
            self.path_cache.clear();
        }

        self.grid.clear();
        self.grid.extend_from_slice(collision_data);
    }

    pub fn update_from_tiles(&mut self, width: u32, height: u32, tiles: &[(u16, u8)]) {
        if self.width != width || self.height != height {
            self.reset();
            self.width = width;
            self.height = height;
            self.grid.reserve_exact(tiles.len());
        } else {
            self.path_cache.clear();
        }

        self.grid.clear();
        for &(_, collision_type) in tiles {
            self.grid.push(collision_type);
        }
    }

    pub fn find_path(
        &mut self,
        from_x: u32,
        from_y: u32,
        to_x: u32,
        to_y: u32,
        has_access: bool,
    ) -> Option<Vec<Node>> {
        let cache_key = (from_x, from_y, to_x, to_y, has_access);
        if let Some(cached_path) = self.path_cache.get(&cache_key) {
            self.cache_hits += 1;
            return cached_path.as_ref().map(|path| {
                path.iter()
                    .map(|&(x, y)| {
                        let index = (y * self.width + x) as usize;
                        let collision_type = self.grid.get(index).copied().unwrap_or(0);
                        Node::new(x, y, collision_type)
                    })
                    .collect()
            });
        }

        self.cache_misses += 1;

        if from_x >= self.width
            || from_y >= self.height
            || to_x >= self.width
            || to_y >= self.height
        {
            self.path_cache.insert(cache_key, None);
            return None;
        }

        let start_index = (from_y * self.width + from_x) as usize;
        let end_index = (to_y * self.width + to_x) as usize;

        if self.is_blocked(start_index, has_access) || self.is_blocked(end_index, has_access) {
            self.path_cache.insert(cache_key, None);
            return None;
        }

        if from_x == to_x && from_y == to_y {
            let result = vec![(from_x, from_y)];
            self.path_cache.insert(cache_key, Some(result.clone()));
            return Some(
                result
                    .into_iter()
                    .map(|(x, y)| {
                        let collision_type = self.grid[start_index];
                        Node::new(x, y, collision_type)
                    })
                    .collect(),
            );
        }

        let mut open_list = BinaryHeap::new();
        let mut came_from = HashMap::with_capacity(256);
        let mut g_scores = HashMap::with_capacity(256);
        let mut closed_set = HashSet::with_capacity(256);

        let start_h = self.calculate_h(from_x, from_y, to_x, to_y);
        let start_node = PathNode::new(from_x, from_y, 0, start_h);

        open_list.push(start_node);
        g_scores.insert((from_x, from_y), 0);

        while let Some(current) = open_list.pop() {
            let current_pos = (current.x, current.y);

            if current.x == to_x && current.y == to_y {
                let path =
                    self.reconstruct_optimized_path(&came_from, current_pos, (from_x, from_y));
                self.path_cache.insert(cache_key, Some(path.clone()));
                return Some(
                    path.into_iter()
                        .map(|(x, y)| {
                            let index = (y * self.width + x) as usize;
                            let collision_type = self.grid.get(index).copied().unwrap_or(0);
                            Node::new(x, y, collision_type)
                        })
                        .collect(),
                );
            }

            if closed_set.contains(&current_pos) {
                continue;
            }

            closed_set.insert(current_pos);

            self.process_neighbors(
                current,
                to_x,
                to_y,
                &mut open_list,
                &mut came_from,
                &mut g_scores,
                &closed_set,
                has_access,
            );
        }

        self.path_cache.insert(cache_key, None);
        None
    }

    #[inline]
    fn movement_cost(&self, from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> u32 {
        let dx = from_x.abs_diff(to_x);
        let dy = from_y.abs_diff(to_y);

        if dx == 1 && dy == 1 {
            14 // Diagonal movement
        } else {
            10 // Orthogonal movement
        }
    }

    #[inline]
    fn calculate_h(&self, from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> u32 {
        let dx = from_x.abs_diff(to_x);
        let dy = from_y.abs_diff(to_y);
        14 * dx.min(dy) + 10 * dx.abs_diff(dy)
    }

    #[inline]
    fn is_blocked(&self, index: usize, has_access: bool) -> bool {
        self.grid.get(index).map_or(true, |&collision_type| {
            // collision_type 1 and 6 are always blocked
            // collision_type 3 (entrance/door) is only blocked if has_access is false
            if collision_type == 1 || collision_type == 6 {
                true
            } else if collision_type == 3 {
                !has_access
            } else {
                false
            }
        })
    }

    #[inline]
    fn is_valid_position(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    fn process_neighbors(
        &self,
        current: PathNode,
        target_x: u32,
        target_y: u32,
        open_list: &mut BinaryHeap<PathNode>,
        came_from: &mut HashMap<(u32, u32), (u32, u32)>,
        g_scores: &mut HashMap<(u32, u32), u32>,
        closed_set: &HashSet<(u32, u32)>,
        has_access: bool,
    ) {
        const DIRECTIONS: [(i32, i32); 8] = [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1), // Orthogonal
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1), // Diagonal
        ];

        for &(dx, dy) in &DIRECTIONS {
            let new_x = current.x as i32 + dx;
            let new_y = current.y as i32 + dy;

            if new_x < 0 || new_y < 0 || new_x >= self.width as i32 || new_y >= self.height as i32 {
                continue;
            }

            let new_x = new_x as u32;
            let new_y = new_y as u32;
            let new_pos = (new_x, new_y);

            if closed_set.contains(&new_pos) {
                continue;
            }

            let index = (new_y * self.width + new_x) as usize;

            if self.is_blocked(index, has_access) {
                continue;
            }

            if dx != 0 && dy != 0 {
                let adj1_x = current.x as i32 + dx;
                let adj1_y = current.y as i32;
                let adj2_x = current.x as i32;
                let adj2_y = current.y as i32 + dy;

                if adj1_x < 0
                    || adj1_x >= self.width as i32
                    || adj2_y < 0
                    || adj2_y >= self.height as i32
                {
                    continue;
                }

                let adj1_index = (adj1_y as u32 * self.width + adj1_x as u32) as usize;
                let adj2_index = (adj2_y as u32 * self.width + adj2_x as u32) as usize;

                if self.is_blocked(adj1_index, has_access) || self.is_blocked(adj2_index, has_access) {
                    continue;
                }
            }

            let tentative_g = current.g + self.movement_cost(current.x, current.y, new_x, new_y);

            // Check if we found a better path
            if let Some(&existing_g) = g_scores.get(&new_pos) {
                if tentative_g >= existing_g {
                    continue;
                }
            }

            // Record the best path
            g_scores.insert(new_pos, tentative_g);
            came_from.insert(new_pos, (current.x, current.y));

            let h = self.calculate_h(new_x, new_y, target_x, target_y);
            let neighbor_node = PathNode::new(new_x, new_y, tentative_g, h);
            open_list.push(neighbor_node);
        }
    }

    fn reconstruct_optimized_path(
        &self,
        came_from: &HashMap<(u32, u32), (u32, u32)>,
        current: (u32, u32),
        start: (u32, u32),
    ) -> Vec<(u32, u32)> {
        let mut path = Vec::new();
        let mut current = current;

        while current != start {
            path.push(current);
            current = match came_from.get(&current) {
                Some(&prev) => prev,
                None => break,
            };
        }

        path.push(start);
        path.reverse();
        path
    }

    pub fn clear_cache(&mut self) {
        self.path_cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }

    pub fn cache_stats(&self) -> (u32, u32, f32) {
        let total = self.cache_hits + self.cache_misses;
        let hit_rate = if total > 0 {
            self.cache_hits as f32 / total as f32 * 100.0
        } else {
            0.0
        };
        (self.cache_hits, self.cache_misses, hit_rate)
    }

    pub fn update_single_tile(&mut self, x: u32, y: u32, collision_type: u8) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if index < self.grid.len() {
                self.grid[index] = collision_type;
                self.path_cache.retain(|&(from_x, from_y, to_x, to_y, _has_access), _| {
                    !((from_x <= x + 1 && from_x + 1 >= x && from_y <= y + 1 && from_y + 1 >= y)
                        || (to_x <= x + 1 && to_x + 1 >= x && to_y <= y + 1 && to_y + 1 >= y))
                });
            }
        }
    }
}
