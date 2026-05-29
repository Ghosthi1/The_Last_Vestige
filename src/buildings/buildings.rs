use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_tilemap::tiles::{TilePos, TileTextureIndex};
use crate::ai::{ FlowFields};
use crate::character::{Colonist};
use crate::map::{Map, TileType};
use crate::components::movement::{GridPosition, Path};
use crate::map::cursor_to_grid;

pub struct BuildingPlugin;
impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TileChangedEvent>();
        app.add_systems(Update, (place_wall_on_click, on_tile_change, on_tile_passability_change));
    }
}

#[derive(Message)]
pub struct TileChangedEvent {
    pub x: u32,
    pub y: u32
}

fn place_wall_on_click( mouse: Res<ButtonInput<MouseButton>>,window: Query<&Window, With<PrimaryWindow>>, camera: Query<(&Camera, &GlobalTransform)>,
                        mut map: ResMut<Map>, mut events: MessageWriter<TileChangedEvent>)
{
    if !mouse.just_pressed(MouseButton::Left) {return}
    let Ok(window) = window.single() else {return};
    let Some(cursor_pos) = window.cursor_position() else {return};
    let Ok((camera, camera_transform)) = camera.single() else { return; };
    let Some((goal_x, goal_y)) = cursor_to_grid(camera, camera_transform, cursor_pos, &map)else { return };

    map.set(goal_x,goal_y ).tile_type = TileType::Wall;
    events.write(TileChangedEvent{x :goal_x,y :goal_y });
}

fn on_tile_passability_change( mut message: MessageReader<TileChangedEvent>, mut flow_field:ResMut<FlowFields>,map: Res<Map>, colonist_grid_pos :Query<&GridPosition, With<Colonist>>,
                               mut vector :Local<Vec<(u32, u32)>>, mut path: Query<&mut Path, With<Colonist>>){
    let events: Vec<_> = message.read().collect();
    if events.is_empty() { return; }
    vector.clear();
    for c in colonist_grid_pos.iter() {
        vector.push(c.0);
    }
    flow_field.colonists.build_flow_fields(&map, &vector);
    for event in events {
        for mut p in path.iter_mut() { if p.0.contains(&(event.x, event.y)) { p.0.clear() } }
    }
}
fn on_tile_change(mut events: MessageReader<TileChangedEvent>, map: Res<Map>, tile_storage_query: Query<&TileStorage>, mut commands: Commands)
{
    let Ok(tile_storage) = tile_storage_query.single() else { return; };
    for event in events.read(){
        let Some(tile_pos) = tile_storage.get(&TilePos {x: event.x, y: event.y}) else { continue; };
        let texture_index = map.get(event.x, event.y).tile_type.texture_index();
        commands.entity(tile_pos).insert(TileTextureIndex(texture_index));
    }
}