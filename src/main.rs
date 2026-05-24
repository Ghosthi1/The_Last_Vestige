mod map;
mod ai;
mod character;
mod constants;
mod enemys;

use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use crate::character::CharacterPlugin;
use crate::constants::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};
use crate::ai::{AiPlugin,FlowFields};
use crate::enemys::EnemyPlugin;

fn main() {
    App::new()
        .insert_resource(map::map_gen::generate_map(MAP_WIDTH, MAP_HEIGHT))
        .insert_resource(map::MapOffset { offset: Vec2::new(-(50.0 * TILE_SIZE/2.0), -(50.0 * TILE_SIZE/2.0)) })
        .insert_resource(FlowFields::default())
        .add_plugins((DefaultPlugins, TilemapPlugin, map::MapRendererPlugin, CharacterPlugin, AiPlugin, EnemyPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}