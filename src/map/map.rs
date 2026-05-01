use bevy::prelude::*;

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
    passable: bool,
}

impl TileType {
    pub fn texture_index(&self) -> u32 {
        match self {
            TileType::Floor => 0,
            TileType::Wall => 1,
            TileType::Door {..} => 2, // ".." I dont care whats in door
        }
    }
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
