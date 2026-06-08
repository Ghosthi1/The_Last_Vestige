// Map //
pub const  TILE_SIZE: f32 = 32.0;
pub const MAP_WIDTH: u32 = 50;
pub const MAP_HEIGHT: u32 = 50;
pub const OFFSETS: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

// ENEMIES //
pub const ENEMY_HEALTH: f32 = 20.0;
pub const ENEMY_SPEED: f32 = 40.0;
pub const ENEMY_STOP_RADIUS: f32 = 0.7;
pub const ENEMY_SEPARATION_STRENGTH: f32 = 10.0;
pub const ENEMY_RANGE: f32 = 1.2 * TILE_SIZE;
pub const ENEMY_DAMAGE: f32 = 20.0;

// Colonists //
pub const COLONIST_HEALTH: f32 = 100.0;
pub const COLONIST_SPEED: f32 = 80.0;
pub const COLONIST_RANGE: f32 = 1.5 * TILE_SIZE;
pub const COLONIST_DAMAGE: f32 = 20.0;