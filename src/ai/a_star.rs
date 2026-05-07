use std::cmp::{max};
use std::collections::{BinaryHeap, HashSet};
use bevy::platform::collections::HashMap;
use crate::map::Map;

/// A position being evaluated during the search, tracking costs to reach it and estimating the remaining distance to the goal.
#[derive(Eq, PartialEq)]
struct Node {
    pos: (u32, u32),
    /// How much it actually cost to get here
    g: u32,
    /// How much it's estimated to cost to get to the goal from here
    h: u32,
}
impl Node {
    /// The total estimated cost of a path
    fn f(&self) -> u32 {
        self.g + self.h
    }
}
// Order the binary heap in lowest to highest
impl Ord for Node {
    fn cmp(&self, other: &Self) ->  std::cmp::Ordering {
        self.f().cmp(&other.f()).reverse()
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Estimates the minimum possible distance between two grid positions, assuming no walls. Using Chebyshev distance.
fn heuristic(a: (u32, u32), b: (u32, u32)) -> u32 {
    max( a.0.abs_diff(b.0),  a.1.abs_diff(b.1))
}

pub fn find_path(map: &Map, start:(u32, u32), goal:(u32, u32)) -> Option<Vec<(u32, u32)>> {
    let mut open_set = BinaryHeap::new();     // The frontier: nodes discovered but not yet fully evaluated
    let mut closed_set = HashSet::new();     // Nodes fully processed (their path is confirmed)
    let mut came_from: HashMap<(u32,u32), (u32,u32)> = HashMap::new();  // The path taken, used to remake the route by working backwards
    let mut g_scores: HashMap<(u32,u32), u32> = HashMap::new();  // The best known cost to reach each position

    open_set.push(Node{pos: start, g:0, h: heuristic(start, goal)});
    g_scores.insert(start, 0);

    while let Some(node) = open_set.pop() {
        // If node pos is the goal get the path taken
        if node.pos == goal{
            let mut path = vec![goal];
            let mut current = goal;

            // Gets the path by getting the tile before from the goal to start
            while let Some(&parent) = came_from.get(&current) {
                current = parent;
                path.push(parent);
            }
            path.reverse();
            return Some(path)
        }

        // Skips nodes that have already been looked at, ensures best route by skipping stale duplicates left in the heap
        if closed_set.contains(&node.pos) {
            continue
        }

        closed_set.insert(node.pos);

        let offsets: [(i32, i32); 8] =[(-1,-1), (0,-1), (1,-1), (-1,0), (1,0), (-1,1), (0,1), (1,1)];
        for (dx, dy) in offsets.iter() {
            let nx: i32 =  node.pos.0 as i32 + *dx;
            let ny: i32 =  node.pos.1 as i32 + *dy;

            if nx >= map.width as i32 || ny >= map.height as i32 || nx < 0 || ny < 0 {continue} // Validation

            let pos = (nx as u32, ny as u32);

            if closed_set.contains(&pos)|| !map.get(pos.0, pos.1).is_passable() {continue}

            let tentative_g = node.g + 1; // Cost to reach neighbour from current node

            // treats the unseen nodes as cost infinity
            if tentative_g < g_scores.get(&pos).copied().unwrap_or(u32::MAX) {
                g_scores.insert(pos, tentative_g);
                came_from.insert(pos, node.pos);
                open_set.push(Node{pos, g: tentative_g, h:  heuristic(pos, goal)});
            }
        }

    }

    None
}