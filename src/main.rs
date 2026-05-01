mod map;

use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

fn main() {
    App::new()
        .insert_resource(map::map_gen::generate_map(50, 50))
        .add_plugins((DefaultPlugins, TilemapPlugin, map::MapRendererPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}