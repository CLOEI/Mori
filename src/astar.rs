use std::{
    cmp::Ordering, collections::{BinaryHeap, HashMap}, vec
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathNode {
    pub f: u32,
    pub g: u32,
    pub h: u32,
    pub x: u32,
    pub y: u32,
    pub parent: Option<(u32, u32)>,
}

impl PathNode {
    pub fn new(x: u32, y: u32, g: u32, h: u32, parent: Option<(u32, u32)>) -> Self {
        Self { f: g + h, g, h, x, y, parent }
    }
}

// ── OpenEntry (stored in BinaryHeap) ────────────────────────────────────────
// Only holds what's needed for sorting — costs + position as key.
// BinaryHeap is a max-heap, so we reverse the ordering to get min-heap behavior.

#[derive(Eq, PartialEq)]
struct OpenEntry {
    f: u32,
    h: u32,
    pos: (u32, u32),
}

impl Ord for OpenEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then(other.h.cmp(&self.h))
    }
}

impl PartialOrd for OpenEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn find_path<F>(
    ox: u32,
    oy: u32,
    tx: u32,
    ty: u32,
    is_passable_cb: F
)
-> Option<Vec<(/*x*/ u32,/*y*/ u32)>>
// direction: (-1. 1) = (Left, Down). for (0,0), just ignore direction check.
// NOTE: this callback must implement bound check, because this function is not
//       dimension aware
where F: Fn(/*x*/ u32, /*y*/ u32, /*direction*/ (i32, i32)) -> bool
{
    if (ox, oy) == (tx, ty) { return Some(vec!()); }
    if !is_passable_cb(ox, oy, (0, 0)) || !is_passable_cb(tx, ty, (0, 0)) { return None }

    let mut open_list: BinaryHeap<OpenEntry> = BinaryHeap::new();
    let mut nodes: HashMap<(u32, u32), PathNode> = HashMap::new();
    let mut closed: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();

    let goal = (tx, ty);
    let start_h = calculate_cost(ox, oy, tx, ty);
    let start_node = PathNode::new(ox, oy, 0, start_h, None);

    open_list.push(OpenEntry { f: start_node.f, h: start_h, pos: (ox, oy) });
    nodes.insert((ox, oy), start_node);

    while let Some(entry) = open_list.pop() {
        let pos = entry.pos;

        if pos == goal {
            return Some(reconstruct_path(&nodes, goal));
        }

        // Skip if already processed (stale entry in heap)
        if closed.contains(&pos) {
            continue;
        }
        closed.insert(pos);

        let current_g = nodes[&pos].g;

        const DIRECTIONS: [(i32, i32); 4] = [
            (-1, 0), (1, 0), (0, -1), (0, 1),
        ];

        for (dx, dy) in DIRECTIONS {
            let nx = pos.0 as i32 + dx;
            let ny = pos.1 as i32 + dy;

            if nx < 0 || ny < 0 {
                continue;
            }

            let nx = nx as u32;
            let ny = ny as u32;

            if !is_passable_cb(nx, ny, (dx, dy)) || closed.contains(&(nx, ny)) {
                continue;
            }

            let new_g = current_g + 1; // move cost is 1
            let new_h = calculate_cost(nx, ny, tx, ty);
            let new_f = new_g + new_h;
            let new_node = PathNode::new(nx, ny, new_g, new_h, Some(pos));

            // Only insert/update if we found a cheaper path
            let should_update = nodes.get(&(nx, ny))
                .map_or(true, |existing| (new_node.f < existing.f) || (new_node.f == existing.f && new_node.h < existing.h));

            if should_update {
                nodes.insert((nx, ny), new_node);
                open_list.push(OpenEntry { f: new_f, h: new_h, pos: (nx, ny) });
            }
        }
    }

    None // no path found
}

fn reconstruct_path(nodes: &HashMap<(u32, u32), PathNode>, goal: (u32, u32)) -> Vec<(u32, u32)> {
    let mut path = Vec::new();
    let mut current = goal;

    loop {
        path.push(current);
        match nodes[&current].parent {
            Some(parent) => current = parent,
            None => break,
        }
    }

    path.reverse();
    path
}

fn calculate_cost(ox: u32, oy: u32, tx: u32, ty: u32) -> u32 {
    ox.abs_diff(tx) + oy.abs_diff(ty)
}
