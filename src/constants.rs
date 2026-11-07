use bevy::math::Vec2;

// region Map //
pub const  TILE_SIZE: Vec2 = Vec2::new(128.0, 112.0);
pub const MAP_WIDTH: u32 = 100;
pub const MAP_HEIGHT: u32 = 100;
pub const OFFSETS: [(i32, i32); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];
// endregion Map //

// region ENEMIES //
pub const ENEMY_HEALTH: f32 = 20.0;
pub const ENEMY_SPEED: f32 = 140.0;
pub const ENEMY_STOP_RADIUS: f32 = 0.7;
pub const ENEMY_SEPARATION_STRENGTH: f32 = 10.0;
pub const ENEMY_RANGE: f32 = 1.2 * TILE_SIZE.x;
pub const ENEMY_DAMAGE: f32 = 20.0;
pub const ENEMY_COUNT: u32 = 0;
pub const ENEMY_SPAWN_EDGE_BAND: u32 = 5;
// endregion ENEMIES //

// region Colonists //
pub const COLONIST_HEALTH: f32 = 40.0;
pub const COLONIST_SPEED: f32 = 120.0;
pub const COLONIST_RANGE: f32 = 1.5 * TILE_SIZE.x;
pub const COLONIST_DAMAGE: f32 = 20.0;
// endregion Colonists //

// region Map Generation //
pub const ROOM_ATTEMPTS: u32 = 100;
pub const ROOM_MIN_SIZE: u32 = 5;
pub const ROOM_MAX_SIZE: u32 = 15;
pub const CORRIDOR_WIDTH: [u32; 3] = [1,3,5];
pub const CORRIDOR_WIDTH_WEIGHTS: [u32; 3] = [2,5,1];
pub const EXTRA_LINKS: u32 = 3;
// endregion Map Generation //

// region Camera //
pub const CAMERA_WASD_SPEED: f32 = TILE_SIZE.x * 8.0;
pub const CAMERA_MAX_ZOOM: f32 = 0.3;
pub const CAMERA_MIN_ZOOM: f32 = 10.0;
pub const CAMERA_ZOOM_SPEED: f32 = 0.1;
// endregion Camera //