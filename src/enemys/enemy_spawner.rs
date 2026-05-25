use bevy::prelude::*;
use crate::character::{GridPosition, Speed};
use crate::constants::{ENEMY_SPEED, TILE_SIZE};
use crate::enemys::Enemy;
use crate::map::Map;

pub struct EnemySpawnerPlugin;
impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy);
    }
}

/// Spawns an Enemy with GridPosition and Speed components
fn spawn_enemy(mut commands: Commands, map: Res<Map>, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("enemeys/Spiders/Grunt.png");
    commands.spawn((
        Enemy,
        GridPosition((25,25)),
        Speed(ENEMY_SPEED),
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            // offset by half map size to match the centred tilemap origin
            25.0 * TILE_SIZE - map.width as f32 * TILE_SIZE/2.0,
            25.0 * TILE_SIZE - map.height as f32 * TILE_SIZE/2.0,
            1.0
        )
    ));
    commands.spawn((
        Enemy,
        GridPosition((27,27)),
        Speed(ENEMY_SPEED),
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            // offset by half map size to match the centred tilemap origin
            27.0 * TILE_SIZE - map.width as f32 * TILE_SIZE/2.0,
            27.0 * TILE_SIZE - map.height as f32 * TILE_SIZE/2.0,
            1.0
        )
    ));
    commands.spawn((
        Enemy,
        GridPosition((30,25)),
        Speed(ENEMY_SPEED),
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            // offset by half map size to match the centred tilemap origin
            30.0 * TILE_SIZE - map.width as f32 * TILE_SIZE/2.0,
            25.0 * TILE_SIZE - map.height as f32 * TILE_SIZE/2.0,
            1.0
        )
    ));

}
