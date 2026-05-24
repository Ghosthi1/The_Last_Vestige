use bevy::prelude::*;
use crate::map::Map;
use crate::constants::{TILE_SIZE, ENEMY_SPEED, ENEMY_STOP_RADIUS, ENEMY_SEPARATION_STRENGTH};
use crate::character::{Colonist, GridPosition, Speed};
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
        app.add_systems(Update, (separate_enemies.before(move_enemy), move_enemy.after(rebuild_colonist_flow_field)));
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

/// Moves towards the next waypoint in the flow field smoothly each frame
fn move_enemy (mut query: Query<(&mut GridPosition,  &mut Transform, &Speed), (With<Enemy>,Without<Colonist>)>,colonist: Query<&Transform,
    (With<Colonist>, Without<Enemy>)>, flow_fields: Res<FlowFields>, map: Res<Map>, time: Res<Time>) {

    let x_offset = map.width as f32 * TILE_SIZE/2.0;
    let y_offset = map.height as f32 * TILE_SIZE/2.0;

    for (mut grid_pos, mut transform, speed) in query.iter_mut() {

        let Some(direction) = flow_fields.colonists.direction_at(grid_pos.0.0, grid_pos.0.1) else { continue };
        if direction == (0,0) {continue;}

        let velocity = Vec2::new(direction.0 as f32, direction.1 as f32).normalize() * speed.0 * time.delta_secs();

        if colonist.iter().any(|colonist| transform.translation.distance_squared(colonist.translation) < (TILE_SIZE * ENEMY_STOP_RADIUS).powi(2)) { continue; }

        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;

        grid_pos.0 = (((transform.translation.x + x_offset) / TILE_SIZE).floor() as u32,  ((transform.translation.y + y_offset) / TILE_SIZE).floor() as u32);

    }
}


/// Pushes the enemies apart so they don't all stack up
fn separate_enemies(mut query: Query<&mut Transform, With<Enemy>> ,time: Res<Time>){
    let positions: Vec<Vec2> = query.iter().map(|t| t.translation.truncate()).collect();
    for (i, mut transform) in query.iter_mut().enumerate() {
        let mut sepaeation = Vec2::ZERO;
        for (j, position) in positions.iter().enumerate() {
            if i == j {continue;}
            let diff = positions[i] - *position;
            let dist = diff.length();

            if dist == 0.0 || dist > TILE_SIZE {continue;}

            sepaeation += diff.normalize() / dist;
        }

        transform.translation.x += sepaeation.x * ENEMY_SEPARATION_STRENGTH * time.delta_secs();
        transform.translation.y += sepaeation.y * ENEMY_SEPARATION_STRENGTH * time.delta_secs();
    }
}
