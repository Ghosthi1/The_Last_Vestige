use std::collections::{BinaryHeap};
use bevy::prelude::*;
use crate::map::Map;
use crate::constants::OFFSETS;
use crate::constants::MAP_WIDTH;
use crate::constants::MAP_HEIGHT;
use std::cmp::Reverse;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
/// What the target of the flow field will be
pub enum FlowLayer {
    Colonist,
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
    directions: Vec<Option<(i8, i8)>>,
    /// Cost tracking buffer reused across rebuilds
    cost_so_far: Vec<u32>,
    valid_goals: Vec<(u32, u32)>,
    open_set: BinaryHeap<(Reverse<u32>, u32, u32)>,
}
impl FlowField{
    /// Creates a new flow field. Needs width and height, and it creates an empty vec for directions
    pub fn new_flow_field(width: u32, height: u32) -> FlowField{
        FlowField{ width, height, directions: vec![None; (width * height) as usize], cost_so_far:vec![u32::MAX; (width * height) as usize],
            open_set: BinaryHeap::new(), valid_goals: Vec::new() }}
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
    pub fn build_flow_fields(&mut self, map: &Map, goals: &[(u32, u32)]){
        self.valid_goals.clear();
        for goal in goals {
            if goal.0 >= self.width || goal.1 >= self.height ||  !map.get(goal.0 , goal.1).is_passable() {continue;} // early return if not valid
            self.valid_goals.push((goal.0 , goal.1));
        }

        if self.valid_goals.is_empty() {return;}

        // Initialization
        self.directions.fill(None);
        self.cost_so_far.fill(u32::MAX);
        self.open_set.clear();

        for goal in &self.valid_goals {
            let goals_index = goal.0 + goal.1 * self.width;
            self.cost_so_far[goals_index as usize] = 0;
            self.directions[goals_index as usize] = Some((0, 0));
            self.open_set.push((Reverse(0u32), goal.0, goal.1));
        };

        while let Some((Reverse(cost), x, y)) = self.open_set.pop() {
            // Cheaper path already found
            if self.cost_so_far[(x + y * self.width) as usize] < cost { continue }

            for (dx, dy) in OFFSETS.iter() {
                let nx: i32 =  x as i32 + *dx; // Neighbour x
                let ny: i32 =  y as i32 + *dy; // Neighbour y

                if nx >= map.width as i32 || ny >= map.height as i32 || nx < 0 || ny < 0 {continue} // Validation
                if  !map.get(nx as u32, ny as u32).is_passable() {continue}

                let move_cost = if *dx != 0 && *dy != 0 { 14 } else { 10 };
                let new_cost = cost + move_cost;

                let ni = (nx + ny * self.width as i32) as usize;

                if new_cost < self.cost_so_far[ni] {
                    self.cost_so_far[ni] = new_cost;
                    self.directions[ni] = Some((x as i8 - nx as i8, y as i8 - ny as i8));
                    self.open_set.push(( Reverse(new_cost),nx as u32,ny as u32));
                }
            }
        }
    }
}

/// Holds all the layers of the FlowFields
#[derive(Resource)]
pub struct FlowFields{
    pub colonists: FlowField,
    pub structures: FlowField,
    pub walls: FlowField
}

impl Default for FlowFields{
    fn default() -> FlowFields{
        FlowFields {
            colonists: FlowField::new_flow_field(MAP_WIDTH, MAP_HEIGHT),
            structures: FlowField::new_flow_field(MAP_WIDTH, MAP_HEIGHT),
            walls: FlowField::new_flow_field(MAP_WIDTH, MAP_HEIGHT),
        }
    }

}

