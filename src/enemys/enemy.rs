use bevy::prelude::*;
use crate::map::Map;
use crate::constants::TILE_SIZE;
use crate::character::{GridPosition, Speed};
use crate::ai::FlowFields;
use crate::ai::ai_plugins::rebuild_colonist_flow_field;

/// Tags the enemy
#[derive(Component)]
pub struct Enemy;

/// Manages Enemy spawning and movement
pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy);
        app.add_systems(Update, move_enemy.after(rebuild_colonist_flow_field));
    }
}

/// Spawns an Enemy with GridPosition and Speed components
fn spawn_enemy(mut commands: Commands, map: Res<Map>, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("enemeys/Spiders/Grunt.png");
    commands.spawn((
        Enemy,
        GridPosition((25,25)),
        Speed(30.0),
        Sprite {
            image: texture,
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            // offset by half map size to match the centred tilemap origin
            25.0 * TILE_SIZE - map.width as f32 * TILE_SIZE/2.0,
            25.0 * TILE_SIZE - map.height as f32 * TILE_SIZE/2.0,
            1.0
        )
    ));}

/// Moves towards the next waypoint in the flow field smoothly each frame
fn move_enemy (mut query: Query<(&mut GridPosition,  &mut Transform, &Speed), With<Enemy>>, flow_fields: Res<FlowFields>, map: Res<Map>, time: Res<Time>) {

    let x_offset = map.width as f32 * TILE_SIZE/2.0;
    let y_offset = map.height as f32 * TILE_SIZE/2.0;

    for (mut grid_pos, mut transform, speed) in query.iter_mut() {

        let Some(direction) = flow_fields.colonists.direction_at(grid_pos.0.0, grid_pos.0.1) else { continue };
        if direction == (0,0) {continue;}

        let nx = (grid_pos.0.0 as i32 + direction.0 as i32) as i32;
        let ny = (grid_pos.0.1 as i32 + direction.1 as i32) as i32;

        let target = Vec3::new(
            nx as f32 * TILE_SIZE - x_offset,
            ny as f32 * TILE_SIZE- y_offset,
            1.0
        );

        transform.translation = transform.translation.move_towards(target, speed.0 * time.delta_secs());

        if transform.translation.distance_squared(target) < 0.01 {
            transform.translation = target;
            grid_pos.0 = (nx as u32, ny as u32);
        }
    }
}
