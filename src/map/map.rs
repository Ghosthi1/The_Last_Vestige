use bevy::post_process::dof::DepthOfFieldMode::Bokeh;
use bevy::prelude::*;
use crate::constants::TILE_SIZE;

/// Represents what a tile fundamentally is. Determines how it looks and how it interacts
#[derive(Default, Clone)]
pub enum TileType {
    #[default]
    Floor,
    Wall,
    /// Determines whether the door is open or closed
    Door {is_open: bool},
}

/// The data attached to a single tile on the map
#[derive(Default, Clone)]
pub struct TileData{
    pub tile_type: TileType,
}

impl TileType {
    /// Gives an index for the tile sheet to assign textures
    pub fn texture_index(&self) -> u32 {
        match self {
            TileType::Floor => 0,
            TileType::Wall => 1,
            TileType::Door {..} => 2, // ".." I dont care whats in door
        }
    }
    /// Determines if a tile is passable based on ots type
    pub fn is_passable(&self) -> bool {
        match self {
            TileType::Floor => true,
            TileType::Wall => false,
            TileType::Door { is_open } => *is_open,
        }
    }
}
/// Used to Ask TileType if its passable
impl TileData {
    pub fn is_passable(&self) -> bool { self.tile_type.is_passable() }
}

/// The entire game map, Owns all the tile data as a flat array, and the dimensions to convert grid coordinates into array positions
#[derive(Resource)]
pub struct Map{
    data: Vec<TileData>,
    pub width: u32,
    pub height: u32,
}
impl Map{
    /// Takes a grid coordinate and returns a read only reference to the tile at that Position
    pub fn get(&self, x:u32, y:u32) -> &TileData{let index = (x + y * self.width) as usize; &self.data[index]}
    /// Takes a grid coordinate and returns a mutable reference so the caller can modify the tile
    pub fn set(&mut self, x:u32, y:u32) -> &mut TileData{let index = (x + y * self.width) as usize; &mut self.data[index]}
    /// Creates a new Map of the given dimensions, with every tile init to its default state.
    pub fn new(width:u32, height:u32) -> Self{
        let size = (width * height) as usize;
        Map {data: vec![TileData::default();size], width, height}
    }
}

/// holds the world space offset
#[derive(Resource)]
pub struct MapOffset{
    pub offset: Vec2,
}

/// Converts a screen-space cursor pos to a map grid coordinate returning None if the cursor is outside the map bounds or the projection fails.
pub fn cursor_to_grid(camera: &Camera,camera_transform: &GlobalTransform, cursor_pos: Vec2, map: &Map) -> Option<(u32, u32)>{
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return None };
    let raw_x = (world_pos.x + map.width as f32 * TILE_SIZE/2.0) / TILE_SIZE;
    let raw_y = (world_pos.y + map.height as f32 * TILE_SIZE/2.0) / TILE_SIZE;
    if !(raw_x >= 0.0 && raw_x < map.width as f32 ||raw_y >= 0.0 && raw_y < map.height as f32) { return None}
    let goal_x = raw_x as u32;
    let goal_y = raw_y as u32;
    Some((goal_x, goal_y))
}