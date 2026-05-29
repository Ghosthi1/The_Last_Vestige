mod map;
mod ai;
mod character;
mod constants;
mod enemys;
mod buildings;
mod components;
mod systems;

use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use crate::character::CharacterPlugin;
use crate::constants::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};
use crate::ai::{AiPlugin,FlowFields};
use crate::buildings::BuildingPlugin;
use crate::enemys::EnemyPlugin;
use crate::enemys::EnemySpawnerPlugin;
use crate::systems::AmbientPlugin;
use crate::systems::CameraPlugin;

fn main() {
    App::new()
        .insert_resource(map::map_gen::generate_map(MAP_WIDTH, MAP_HEIGHT))
        .insert_resource(map::MapOffset { offset: Vec2::new(-( MAP_WIDTH as f32 * TILE_SIZE/2.0), -(MAP_HEIGHT as f32* TILE_SIZE/2.0)) })
        .insert_resource(FlowFields::default())
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), TilemapPlugin))
        .add_plugins((EnemySpawnerPlugin, map::MapRendererPlugin, CharacterPlugin, AiPlugin, EnemyPlugin, CameraPlugin, BuildingPlugin, AmbientPlugin))
        .add_systems(Update, debug)
        .run();
}

////////////////////////////////////////
/// Creates Lines so the grid is visible
///////////////////////////////////////
fn debug(mut gizmos: Gizmos){
    for i in 0..=MAP_WIDTH{
        let x = -(MAP_WIDTH as f32 * TILE_SIZE / 2.0) + i as f32 * TILE_SIZE;
        gizmos.line_2d( Vec2::new(x, -(MAP_HEIGHT as f32 * TILE_SIZE / 2.0)),  Vec2::new(x, MAP_HEIGHT as f32 * TILE_SIZE / 2.0), Color::WHITE)
    }
    for j in 0..=MAP_HEIGHT{
        let y = -(MAP_HEIGHT as f32 * TILE_SIZE / 2.0) + j as f32 * TILE_SIZE;
        gizmos.line_2d(  Vec2::new(-(MAP_WIDTH as f32 * TILE_SIZE / 2.0), y), Vec2::new(MAP_WIDTH as f32 * TILE_SIZE / 2.0, y), Color::WHITE)
    }
}