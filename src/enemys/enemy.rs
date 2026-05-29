use bevy::prelude::*;
use crate::map::{Map};
use crate::constants::{TILE_SIZE, ENEMY_STOP_RADIUS, ENEMY_SEPARATION_STRENGTH, OFFSETS, MAP_WIDTH, MAP_HEIGHT};
use crate::character::{Colonist};
use crate::ai::FlowFields;
use crate::ai::ai_plugins::rebuild_colonist_flow_field;
use crate::components::movement::{GridPosition, Speed};

/// Tags the enemy
#[derive(Component)]
pub struct Enemy;

/// Manages Enemy spawning and movement
pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (separate_enemies.before(move_enemy), move_enemy.after(rebuild_colonist_flow_field)));
    }
}

/// Moves towards the next waypoint in the flow field smoothly each frame
fn move_enemy (mut query: Query<(&mut GridPosition,  &mut Transform, &Speed), (With<Enemy>,Without<Colonist>)>,colonist: Query<&Transform,
    (With<Colonist>, Without<Enemy>)>, flow_fields: Res<FlowFields>, map: Res<Map>, time: Res<Time>) {

    let x_offset = map.width as f32 * TILE_SIZE/2.0;
    let y_offset = map.height as f32 * TILE_SIZE/2.0;

    let colonist_positions: Vec<Vec2> = colonist.iter().map(|t| t.translation.truncate()).collect();

    for (mut grid_pos, mut transform, speed) in query.iter_mut() {

        let dir_option = flow_fields.colonists.direction_at(grid_pos.0.0, grid_pos.0.1);

        match dir_option{
            Some(direction) => {
                if direction == (0,0) {continue;}

                let velocity = Vec2::new(direction.0 as f32, direction.1 as f32).normalize() * speed.0 * time.delta_secs();

                if colonist_positions.iter().any(|pos| transform.translation.truncate().distance_squared(*pos) < (TILE_SIZE * ENEMY_STOP_RADIUS).powi(2)) { continue; }

                if tile_at( transform.translation.truncate() + Vec2::new(velocity.x, 0.0), &map).is_some() {
                    transform.translation.x += velocity.x
                }
                if tile_at( transform.translation.truncate() + Vec2::new(0.0, velocity.y), &map).is_some() {
                    transform.translation.y += velocity.y;
                }

                grid_pos.0 = (((transform.translation.x + x_offset) / TILE_SIZE).floor() as u32,  ((transform.translation.y + y_offset) / TILE_SIZE).floor() as u32);
            }
            None => {
                for offset in OFFSETS{
                    let nx = grid_pos.0.0 as i32 + offset.0;
                    let ny = grid_pos.0.1 as i32 + offset.1;

                    if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32{ continue; };

                    if let Some(_direction) = flow_fields.colonists.direction_at(nx as u32, ny as u32){
                        let velocity =  Vec2::new(offset.0 as f32, offset.1 as f32).normalize() * speed.0 * time.delta_secs();
                        if tile_at( transform.translation.truncate() + Vec2::new(velocity.x, 0.0), &map).is_some() {
                            transform.translation.x += velocity.x
                        }
                        if tile_at( transform.translation.truncate() + Vec2::new(0.0, velocity.y), &map).is_some() {
                            transform.translation.y += velocity.y;
                        }

                        grid_pos.0 = (((transform.translation.x + x_offset) / TILE_SIZE).floor() as u32,  ((transform.translation.y + y_offset) / TILE_SIZE).floor() as u32);
                        break;
                    }

                }
            }
        }
    }
}

/// Converts World space into tile coordinates, Returning none if Impassable
fn tile_at(world_pos: Vec2, map: &Map) -> Option<(u32, u32)>{
    let raw_x = ((world_pos.x + (MAP_WIDTH as f32 * TILE_SIZE / 2.0)) / TILE_SIZE ).floor();
    let raw_y = ((world_pos.y + (MAP_HEIGHT as f32 * TILE_SIZE / 2.0)) / TILE_SIZE).floor();
    if raw_y < 0.0 ||raw_x < 0.0 || raw_y >= MAP_HEIGHT as f32|| raw_x >= MAP_WIDTH as f32{return None;}
    let tx = raw_x as u32;
    let ty = raw_y as u32;
    if !map.get(tx,ty).tile_type.is_passable() { return None; };
    Some((tx, ty))
}


/// Pushes the enemies apart so they don't all stack up
fn separate_enemies(mut query: Query<&mut Transform, With<Enemy>> ,time: Res<Time>, map: Res<Map>){
    let positions: Vec<Vec2> = query.iter().map(|t| t.translation.truncate()).collect();
    let mut forces: Vec<Vec2> = vec![Vec2::ZERO; positions.len()];

    for i in 0..positions.len() {
        for j in (i+1)..positions.len() {
            let diff = positions[i] - positions[j];
            let dist = diff.length();

            if dist == 0.0 || dist > TILE_SIZE {continue;}

            forces[i] += (1.0 - dist / TILE_SIZE) * diff / dist;
            forces[j] -= (1.0 - dist / TILE_SIZE) * diff / dist;

        }
    }
    for (i, mut transform) in query.iter_mut().enumerate() {
        let x_delta = forces[i].x * ENEMY_SEPARATION_STRENGTH * time.delta_secs();
        if tile_at( transform.translation.truncate() + Vec2::new(x_delta, 0.0), &map).is_some() {
            transform.translation.x += x_delta;
        }

        let y_delta = forces[i].y * ENEMY_SEPARATION_STRENGTH * time.delta_secs();
        if tile_at( transform.translation.truncate() + Vec2::new(0.0, y_delta), &map).is_some() {
            transform.translation.y += y_delta;
        }
    }

}
