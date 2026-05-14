use std::cmp::{max};
use std::collections::{BinaryHeap};

use crate::map::Map;

/// A position being evaluated during the search, tracking costs to reach it and estimating the remaining distance to the goal.
#[derive(Eq, PartialEq)]
struct Node {
    pos: (u32, u32),
    /// How much it actually cost to get here
    g: u32,
    /// How much it's estimated to cost to get to the goal from here
    h: u32,
    f: u32,
}
// Order the binary heap in lowest to highest
impl Ord for Node {
    fn cmp(&self, other: &Self) ->  std::cmp::Ordering {
        self.f.cmp(&other.f)
            .then(self.h.cmp(&other.h))
            .reverse()

    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Estimates the minimum possible distance between two grid positions, assuming no walls. Using Chebyshev distance.
fn heuristic(a: (u32, u32), b: (u32, u32)) -> u32 {
    max( a.0.abs_diff(b.0),  a.1.abs_diff(b.1)) * 10
}

/// finds the best path between two points using A*
pub fn find_path(map: &Map, start:(u32, u32), goal:(u32, u32)) -> Option<Vec<(u32, u32)>> {

    // Early exit of no movement needed
    if start == goal { return Some(vec![start]); }
    // Validation checker if in bounds
    if start.0 >= map.width || start.1 >= map.height || goal.0 >= map.width || goal.1 >= map.height {return None}
    if !map.get(goal.0, goal.1).is_passable() {return None}

    let mut open_set = BinaryHeap::new();     // The frontier: nodes discovered but not yet fully evaluated
    let mut closed_set =  vec![false; (map.width * map.height) as usize];     // Nodes fully processed (their path is confirmed)
    let mut came_from = vec![u32::MAX; (map.width * map.height) as usize];  // The path taken, used to remake the route by working backwards
    let mut g_scores = vec![u32::MAX; (map.width * map.height) as usize];    // The best known cost to reach each position

    const OFFSETS: [(i32, i32); 8] =[(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

    let h = heuristic(start, goal);
    open_set.push(Node{pos: start, g:0, h, f: h});
    g_scores[start.0 as usize + start.1 as usize * map.width as usize] = 0;

    while let Some(node) = open_set.pop() {
        // If node pos is the goal get the path taken
        if node.pos == goal{
            let mut path = vec![goal];
            let mut current = goal;

            // Gets the path by getting the tile before from the goal to start
            loop {
                let current_idx = current.0 as usize + current.1 as usize * map.width as usize;
                let parent_idx = came_from[current_idx];
                if parent_idx == u32::MAX { break }
                let x = parent_idx % map.width;
                let y = parent_idx / map.width;
                current = (x, y);
                path.push(current);
            }
            path.reverse();
            return Some(path)
        }

        let idx = node.pos.0 as usize + node.pos.1 as usize * map.width as usize;

        // Skips nodes that have already been looked at, ensures best route by skipping stale duplicates left in the heap
        if  closed_set[idx]
        {
            continue
        }

        closed_set[idx] = true;

        for (dx, dy) in OFFSETS.iter() {
            let nx: i32 =  node.pos.0 as i32 + *dx;
            let ny: i32 =  node.pos.1 as i32 + *dy;

            if nx >= map.width as i32 || ny >= map.height as i32 || nx < 0 || ny < 0 {continue} // Validation

            let pos = (nx as u32, ny as u32);
            let neighbour_idx = pos.0 as usize + pos.1 as usize * map.width as usize;

            if  closed_set[neighbour_idx]|| !map.get(pos.0, pos.1).is_passable() {continue}

            let move_cost = if *dx != 0 && *dy != 0 { 14 } else { 10 };
            let tentative_g = node.g + move_cost; // Cost to reach neighbor from current node

            // treats the unseen nodes as cost infinity
            if tentative_g <  g_scores[neighbour_idx]   {
                g_scores[neighbour_idx] =tentative_g;
                came_from[neighbour_idx] = (node.pos.0 as usize + node.pos.1 as usize * map.width as usize) as u32;
                let h = heuristic(pos, goal);
                open_set.push(Node{pos, g: tentative_g, h, f: tentative_g + h});
            }
        }

    }
    None
}