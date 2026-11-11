use crate::constants::{OFFSETS, TILE_SIZE};
use bevy::prelude::*;

pub(crate) const WALL_NEIGHBOURS:[(i32,i32,u32); 4] = [(0, 1, 1), (1, 0, 2), (0, -1, 4), (-1, 0, 8)];
const WALL_SHEET_START: u32 = 6;

/// Represents what a tile fundamentally is. Determines how it looks and how it interacts
#[derive(Default, Clone)]
pub enum TileType {
    #[default]
    Floor,
    Wall,
    /// Determines whether the door is open or closed
    Door {
        is_open: bool,
    },
}

/// The data attached to a single tile on the map
#[derive(Default, Clone)]
pub struct TileData {
    pub tile_type: TileType,
    /// what floors to use
    pub floor_variant: u32,
}

impl TileType {
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
    pub fn is_passable(&self) -> bool {
        self.tile_type.is_passable()
    }

    /// Gives an index for the tile sheet to assign textures
    pub fn texture_index(&self) -> u32 {
        match self.tile_type {
            TileType::Floor => self.floor_variant,
            TileType::Wall => 6,
            TileType::Door { is_open } => {
                if is_open {
                    1
                } else {
                    1
                }
            }
        }
    }
}

/// The entire game map, Owns all the tile data as a flat array, and the dimensions to convert grid coordinates into array positions
#[derive(Resource)]
pub struct Map {
    data: Vec<TileData>,
    pub width: u32,
    pub height: u32,
}
impl Map {
    /// Takes a grid coordinate and returns a read only reference to the tile at that Position
    pub fn get(&self, x: u32, y: u32) -> &TileData {
        let index = (x + y * self.width) as usize;
        &self.data[index]
    }
    /// Takes a grid coordinate and returns a mutable reference so the caller can modify the tile
    pub fn set(&mut self, x: u32, y: u32) -> &mut TileData {
        let index = (x + y * self.width) as usize;
        &mut self.data[index]
    }
    /// Creates a new Map of the given dimensions, with every tile init to its default state.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Map {
            data: vec![TileData::default(); size],
            width,
            height,
        }
    }
    /// Converts a grid coordinate to a world space position, with the centre of the tile
    pub fn grid_to_world(&self, x: u32, y: u32) -> Vec2 {
        Vec2::new(
            (x as f32 + 0.5) * TILE_SIZE.x - (self.width as f32 * TILE_SIZE.x / 2.0),
            (y as f32 + 0.5) * TILE_SIZE.y - (self.height as f32 * TILE_SIZE.y / 2.0),
        )
    }
    /// Converts a world space position to a grid coordinate, returning None if the position is outside the map bounds
    pub fn world_to_grid(&self, world_pos: Vec2) -> Option<(u32, u32)> {
        let raw_x = ((world_pos.x + self.width as f32 * TILE_SIZE.x / 2.0) / TILE_SIZE.x).floor();
        let raw_y = ((world_pos.y + self.height as f32 * TILE_SIZE.y / 2.0) / TILE_SIZE.y).floor();
        if !(raw_x >= 0.0
            && raw_x < self.width as f32
            && raw_y >= 0.0
            && raw_y < self.height as f32)
        {
            return None;
        }
        let goal_x = raw_x as u32;
        let goal_y = raw_y as u32;
        Some((goal_x, goal_y))
    }

    pub fn tile_texture_index(&self, x: u32, y: u32) -> u32 {
        let tile = self.get(x, y);
        if !matches!(tile.tile_type, TileType::Wall) {
            return tile.texture_index();
        }
        let xi = x as i32;
        let yi = y as i32;
        let mut mask: u32 = 0;

        for &(dx, dy, bit) in WALL_NEIGHBOURS.iter() {
            if self.is_wall_at(xi + dx, yi + dy){
                mask |= bit;
            }
        }
        WALL_SHEET_START + mask
    }

    fn is_wall_at(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return true
        }
        matches!(self.get(x as u32, y as u32).tile_type, TileType::Wall)
    }
}


/// Converts a screen-space cursor pos to a map grid coordinate returning None if the cursor is outside the map bounds or the projection fails.
pub fn cursor_to_grid(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_pos: Vec2,
    map: &Map,
) -> Option<(u32, u32)> {
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return None;
    };
    map.world_to_grid(world_pos)
}
