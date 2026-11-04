mod map;
mod ai;
mod colonists;
mod constants;
mod enemys;
mod buildings;
mod components;
mod systems;
mod combat;
mod death;
mod ui;
mod animation;
mod debugs;

use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use crate::colonists::CharacterPlugin;
use crate::constants::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};
use crate::ai::{AiPlugin,FlowFields};
use crate::buildings::BuildingPlugin;
use crate::enemys::EnemyPlugin;
use crate::enemys::EnemySpawnerPlugin;
use crate::systems::AmbientPlugin;
use crate::systems::CameraPlugin;
use crate::colonists::ColonistSpawnerPlugin;
use crate::colonists::SelectionPlugin;
use crate::combat::CombatPlugin;
use crate::death::DeathPlugin;
use crate::ui::life_support::LifeSupportPlugin;
use crate::ui::UiPlugin;
use crate::animation::AnimationPlugin;
use crate::animation::ColonistAnimationPlugin;
use crate::debugs::DebugPlugin;

fn main() {
    App::new()
        .insert_resource(map::map_gen::generate_map(MAP_WIDTH, MAP_HEIGHT))
        .insert_resource(map::MapOffset { offset: Vec2::new(-( MAP_WIDTH as f32 * TILE_SIZE/2.0), -(MAP_HEIGHT as f32* TILE_SIZE/2.0)) })
        .insert_resource(FlowFields::default())
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), TilemapPlugin))
        .add_plugins((EnemySpawnerPlugin, ColonistSpawnerPlugin, map::MapRendererPlugin, CharacterPlugin,
                      AiPlugin, EnemyPlugin, CameraPlugin, BuildingPlugin, AmbientPlugin, SelectionPlugin, CombatPlugin, DeathPlugin,
                      UiPlugin,LifeSupportPlugin, AnimationPlugin))
        .add_plugins((ColonistAnimationPlugin))

        // Debug //
        .add_plugins(DebugPlugin)

        .run();
}