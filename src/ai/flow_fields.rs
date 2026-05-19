use std::collections::{BinaryHeap, HashMap};
use bevy::prelude::*;
use crate::map::Map;
use crate::constants::OFFSETS;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
/// What the target of the flow field will be
pub enum FlowLayer {
    Colonists,
    Structures,
    Walls,
}

/// A precomputed directional map guiding entities toward a goal across the tile grid.
pub struct FlowField{
    /// The Width of the FlowField
    width: u32,
    /// The Height of the FlowField
    height: u32,
    /// The direction the tile is pointing towards the goal: None if tile can't reach the goal
    directions: Vec<Option<(i8, i8)>>
}
impl FlowField{
    /// Creates a new flow field. Needs width and height, and it creates an empty vec for directions
    pub fn new_flow_field(width: u32, height: u32) -> FlowField{ FlowField{ width, height, directions: vec![None; (width * height) as usize]}}
    /// Gets the direction a tile is pointing
    pub fn direction_at(&self, x: u32, y: u32) -> Option<(i8, i8)>{
        // If x,y is out of bounds
        if x >= self.width || y >= self.height {None}
        else {
            let index = (x + y * self.width) as usize;
            // lookup the index in the vec
            self.directions[index]
        }
    }

    /// Runs Dijkstra outward from the goal tile, filling each reachable tile with a direction pointing toward the goal.
    pub fn build_flow_fields(&mut self, map: &Map, goal: (u32,u32)){
        if goal.0 >= self.width || goal.1 >= self.height ||  !map.get(goal.0 , goal.1).is_passable() {return;} // early return if not valid


        // Initialization
        self.directions = vec![None; (self.width * self.height) as usize];
        let mut cost_so_far = vec![u32::MAX; (self.width * self.height) as usize];
        let goals_index = goal.0 + goal.1 * self.width;
        cost_so_far[goals_index as usize] = 0;
        self.directions[goals_index as usize] = Some((0,0));
        let mut open_set = BinaryHeap::new();
        open_set.push((0u32, goal.0, goal.1));

        while let Some((cost, x, y)) = open_set.pop() {
            // Cheaper path already found
            if cost_so_far[(x + y * self.width) as usize] < cost { continue }

            for (dx, dy) in OFFSETS.iter() {
                let nx: i32 =  x as i32 + *dx; // Neighbour x
                let ny: i32 =  y as i32 + *dy; // Neighbour y

                if nx >= map.width as i32 || ny >= map.height as i32 || nx < 0 || ny < 0 {continue} // Validation
                if  !map.get(nx as u32, ny as u32).is_passable() {continue}

                let move_cost = if *dx != 0 && *dy != 0 { 14 } else { 10 };
                let new_cost = cost + move_cost;

                if new_cost < cost_so_far[(nx + ny * self.width as i32) as usize] {
                    cost_so_far[(nx + ny * self.width as i32) as usize] = new_cost;
                    self.directions[(nx + ny * self.width as i32) as usize] = Some((x as i8 - nx as i8, y as i8 - ny as i8));
                    open_set.push((new_cost,nx as u32,ny as u32));
                }
            }
        }
    }
}

/// Holds all the layers of the FlowFields
#[derive(Resource)]
pub struct FlowFields{
    pub layers: HashMap<FlowLayer, FlowField>
}

