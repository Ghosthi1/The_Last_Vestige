// Map //
pub const  TILE_SIZE: f32 = 128.0;
pub const MAP_WIDTH: u32 = 50;
pub const MAP_HEIGHT: u32 = 50;
pub const OFFSETS: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

// ENEMIES //
pub const ENEMY_HEALTH: f32 = 20.0;
pub const ENEMY_SPEED: f32 = 140.0;
pub const ENEMY_STOP_RADIUS: f32 = 0.7;
pub const ENEMY_SEPARATION_STRENGTH: f32 = 10.0;
pub const ENEMY_RANGE: f32 = 1.2 * TILE_SIZE;
pub const ENEMY_DAMAGE: f32 = 20.0;
pub const ENEMY_COUNT: u32 = 500;
pub const ENEMY_SPAWN_EDGE_BAND: u32 = 5;

// Colonists //
pub const COLONIST_HEALTH: f32 = 40.0;
pub const COLONIST_SPEED: f32 = 120.0;
pub const COLONIST_RANGE: f32 = 1.5 * TILE_SIZE;
pub const COLONIST_DAMAGE: f32 = 20.0;

// Camera //
pub const CAMERA_WASD_SPEED: f32 = TILE_SIZE * 8.0;
pub const CAMERA_MAX_ZOOM: f32 = 0.3;
pub const CAMERA_MIN_ZOOM: f32 = 3.0;
pub const CAMERA_ZOOM_SPEED: f32 = 0.1;