use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_tilemap::tiles::{TilePos, TileTextureIndex};
use crate::ai::{ FlowFields};
use crate::character::{Colonist};
use crate::constants::TILE_SIZE;
use crate::map::{Map, TileType};
use crate::components::movement::{GridPosition, Path};

pub struct BuildingPlugin;
impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TileChangedEvent>();
        app.add_systems(Update, (place_wall_on_click, on_tile_change));
    }
}

#[derive(Message)]
pub struct TileChangedEvent {
    pub x: u32,
    pub y: u32
}

fn place_wall_on_click( mouse: Res<ButtonInput<MouseButton>>,window: Query<&Window, With<PrimaryWindow>>, camera: Query<(&Camera, &GlobalTransform)>,
                        mut map: ResMut<Map>, mut events: MessageWriter<TileChangedEvent>,mut flow_fields: ResMut<FlowFields>,
                        colonist_pos: Query<&GridPosition, With<Colonist>>, mut positions: Local<Vec<(u32, u32)>>,mut colonist_paths: Query<&mut Path, With<Colonist>>)
{
    if !mouse.just_pressed(MouseButton::Left) {return}
    let Ok(window) = window.single() else {return};
    let Some(cursor_pos) = window.cursor_position() else {return};
    let Ok((camera, camera_transform)) = camera.single() else { return; };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };

    let raw_x = (world_pos.x + map.width as f32 * TILE_SIZE/2.0) / TILE_SIZE;
    let raw_y = (world_pos.y + map.height as f32 * TILE_SIZE/2.0) / TILE_SIZE;
    if !(raw_x >= 0.0 && raw_x < map.width as f32) { return }
    if !(raw_y >= 0.0 && raw_y < map.height as f32) {return }
    let goal_x = raw_x as u32;
    let goal_y = raw_y as u32;

    map.set(goal_x,goal_y ).tile_type = TileType::Wall;

    positions.clear();

    for grid_pos in colonist_pos.iter() {
        positions.push(grid_pos.0);
    }
    flow_fields.colonists.build_flow_fields(&map, &positions);

    for mut path in colonist_paths.iter_mut(){
        if path.0.contains(&(goal_x,goal_y)){path.0.clear()};
    }

    events.write(TileChangedEvent{x :goal_x,y :goal_y });
}

fn on_tile_change(mut events: MessageReader<TileChangedEvent>, map: Res<Map>, tile_storage_query: Query<&TileStorage>, mut commands: Commands)
{
    for event in events.read(){
        let Ok(tile_storage) = tile_storage_query.single() else { return; };
        let Some(tile_pos) = tile_storage.get(&TilePos {x: event.x, y: event.y}) else { continue; };
        let texture_index = map.get(event.x, event.y).tile_type.texture_index();
        commands.entity(tile_pos).insert(TileTextureIndex(texture_index));
    }
}