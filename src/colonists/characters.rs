use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::map::Map;
use crate::ai::a_star::find_path;
use crate::constants::{ENEMY_SEPARATION_STRENGTH, MAP_HEIGHT, MAP_WIDTH, OFFSETS, TILE_SIZE};
use crate::components::movement::{GridPosition, Path, Speed};
use crate::map::cursor_to_grid;
use std::collections::HashSet;
use bevy::reflect::Set;

/// Tags the colonists
#[derive(Component)]
pub struct Colonist;

/// Manages colonists spawning and movement
pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,  (separate_colonists.before(move_character), move_to_click.before(move_character), move_character));
    }
}

/// Moves towards the next waypoint in the path smoothly each frame
fn move_character (time: Res<Time>,map: Res<Map>, mut query: Query<(&mut GridPosition, &mut Path, &mut Transform, &Speed)>) {

    let mut occupied: HashSet<(u32, u32)> = query.iter().map(|(grid_pos, _,_,_)|  grid_pos.0).collect();

    let x_offset = map.width as f32 * TILE_SIZE/2.0;
    let y_offset = map.height as f32 * TILE_SIZE/2.0;

    for (mut grid_pos, mut path, mut transform, speed) in query.iter_mut() {
        let Some(next) = path.0.front() else { continue };

        let target =   Vec3::new(
            next.0 as f32 * TILE_SIZE + TILE_SIZE / 2.0 - x_offset,
            next.1 as f32 * TILE_SIZE + TILE_SIZE / 2.0 - y_offset,
            1.0
        );

        transform.translation = transform.translation.move_towards(target, speed.0 * time.delta_secs());

        if transform.translation.distance_squared(target) < 0.01 {
            if occupied.contains(next) && *next != grid_pos.0{
                let mut free_neighbor: Option<(u32, u32)> = None;
                for (dx,dy) in OFFSETS{
                    let neighbor = (next.0 as i32 + dx , next.1 as i32 + dy);
                    let neighbor_u32: (u32, u32);
                    if neighbor.0 < 0 || neighbor.1 < 0{
                        continue;
                    }
                    neighbor_u32 = (neighbor.0 as u32 , neighbor.1 as u32);
                    if neighbor_u32.0 >= map.width || neighbor_u32.1 >= map.height{
                        continue
                    }
                    if map.get(neighbor_u32.0, neighbor_u32.1).tile_type.is_passable() && !occupied.contains(&neighbor_u32){
                        free_neighbor = Some(neighbor_u32);
                        break
                    }
                }
                if let Some(free_neighbor) = free_neighbor {
                    path.0[0] = free_neighbor;
                    occupied.insert(free_neighbor);
                }
            }
            else {
                transform.translation = target;
                occupied.insert(*next);
                grid_pos.0 = *next;
                path.0.pop_front();
            }
        }
    }
}

/// Gets the clicked tile and sets the characters path to that
fn move_to_click(mouse: Res<ButtonInput<MouseButton>>,
                 window: Query<&Window, With<PrimaryWindow>>,
                 camera: Query<(&Camera, &GlobalTransform)>,
                 map: Res<Map>,
                 mut characters: Query<(&GridPosition, &mut Path)> )
{
    if !mouse.just_pressed(MouseButton::Right) {return}
    let Ok(window) = window.single() else {return};
    let Some(cursor_pos) = window.cursor_position() else {return};
    let Ok((camera, camera_transform)) = camera.single() else { return; };
    let Some((goal_x, goal_y)) = cursor_to_grid(camera, camera_transform, cursor_pos, &map)else { return };
    let goal = (goal_x, goal_y);

    let mut occupied: HashSet<(u32, u32)> = characters.iter().map(|(grid_pos, path)|  grid_pos.0).collect();

    for (grid_pos, mut path) in characters.iter_mut() {
        let start = path.0.front().copied().unwrap_or(grid_pos.0);
        let mut free_neighbor: Option<(u32, u32)> = None;

        if occupied.contains(&goal){
            for (dx,dy) in OFFSETS{
                let neighbor = (goal.0 as i32 + dx , goal.1 as i32 + dy);
                let neighbor_u32: (u32, u32);
                if neighbor.0 < 0 || neighbor.1 < 0{
                    continue;
                }
                neighbor_u32 = (neighbor.0 as u32 , neighbor.1 as u32);
                if neighbor_u32.0 >= map.width || neighbor_u32.1 >= map.height{
                    continue
                }
                if map.get(neighbor_u32.0, neighbor_u32.1).tile_type.is_passable() && !occupied.contains(&neighbor_u32){
                    free_neighbor = Some(neighbor_u32);
                    break
                }
            }
        }
        let actual_goal = free_neighbor.unwrap_or(goal);
        path.0 = find_path(&map, start, actual_goal).unwrap_or_default().into();
        occupied.insert(actual_goal);
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
fn separate_colonists(mut query: Query<&mut Transform, With<Colonist>> ,time: Res<Time>, map: Res<Map>){
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
        if tile_at(transform.translation.truncate() + Vec2::new(x_delta, 0.0), &map).is_some() {
            transform.translation.x += x_delta;
        }

        let y_delta = forces[i].y * ENEMY_SEPARATION_STRENGTH * time.delta_secs();
        if tile_at(transform.translation.truncate() + Vec2::new(0.0, y_delta), &map).is_some() {
            transform.translation.y += y_delta;
        }
    }
}
