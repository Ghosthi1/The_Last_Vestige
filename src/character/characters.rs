use bevy::prelude::*;
use std::collections::VecDeque;
use bevy::window::PrimaryWindow;
use crate::map::Map;
use crate::ai::a_star::find_path;
use crate::constants::TILE_SIZE;

/// Current tile coordinates
#[derive(Component)]
pub struct GridPosition(pub(u32, u32));

/// Path to the target
#[derive(Component)]
pub struct Path (pub VecDeque<(u32,u32)>);

/// The movement speed in tiles per second
#[derive(Component)]
pub struct Speed (pub f32);

/// Tags the character
#[derive(Component)]
pub struct Colonist;

/// Manages character spawning and movement
pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_character);
        app.add_systems(Update, (move_to_click,move_character ).chain());
    }
}

/// Spawns a character with GridPosition, Path and Speed components
fn spawn_character(mut commands: Commands, map: Res<Map>, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("Colonists/Knight/Knight_1.png");
    commands.spawn((
        Colonist,
        GridPosition((1,1)),
        Path(VecDeque::new()),
        Speed(50.0),
        Sprite {
            image: texture,
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
            },
        Transform::from_xyz(
            // offset by half map size to match the centred tilemap origin
            1.0 * TILE_SIZE - map.width as f32 * TILE_SIZE/2.0,
            1.0 * TILE_SIZE - map.height as f32 * TILE_SIZE/2.0,
            1.0
        )
    ));}

/// Moves towards the next waypoint in the path smoothly each frame
fn move_character (time: Res<Time>,map: Res<Map>, mut query: Query<(&mut GridPosition, &mut Path, &mut Transform, &Speed)>) {

    let x_offset = map.width as f32 * TILE_SIZE/2.0;
    let y_offset = map.height as f32 * TILE_SIZE/2.0;

    for (mut grid_pos, mut path, mut transform, speed) in query.iter_mut() {
        let Some(next) = path.0.front() else { continue };

        let target =   Vec3::new(
            next.0 as f32 * TILE_SIZE - x_offset,
            next.1 as f32 * TILE_SIZE- y_offset,
            1.0
        );

        transform.translation = transform.translation.move_towards(target, speed.0 * time.delta_secs());

        if transform.translation.distance_squared(target) < 0.01 {
            transform.translation = target;
            grid_pos.0 = *next;
            path.0.pop_front();
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
    if !mouse.just_pressed(MouseButton::Left) {return}
    let Ok(window) = window.single() else {return};
    let Some(cursor_pos) = window.cursor_position() else {return};
    let Ok((camera, camera_transform)) = camera.single() else { return; };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };

    // add half map size to convert from centred world space back to grid coordinates
    let raw_x = ((world_pos.x + map.width as f32 * TILE_SIZE/2.0) / TILE_SIZE);
    let raw_y = ((world_pos.y + map.height as f32 * TILE_SIZE/2.0) / TILE_SIZE);
    if !(raw_x >= 0.0 && raw_x < map.width as f32) { return }
    if !(raw_y >= 0.0 && raw_y < map.height as f32) {return }
    let goal_x = raw_x as u32;
    let goal_y = raw_y as u32;
    let goal = (goal_x, goal_y);


    for (grid_pos, mut path) in characters.iter_mut() {
        let new_path = find_path(&map, grid_pos.0, goal).unwrap_or_default().into();
        path.0 = new_path;
    }
}