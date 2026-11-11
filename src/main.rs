mod ai;
mod animation;
mod buildings;
mod colonists;
mod combat;
mod components;
mod constants;
mod death;
mod debugs;
mod enemys;
mod map;
mod systems;
mod ui;

use crate::ai::{AiPlugin, FlowFields};
use crate::animation::AnimationPlugin;
use crate::animation::ColonistAnimationPlugin;
use crate::buildings::BuildingPlugin;
use crate::colonists::CharacterPlugin;
use crate::colonists::ColonistSpawnerPlugin;
use crate::colonists::SelectionPlugin;
use crate::combat::CombatPlugin;
use crate::constants::{MAP_HEIGHT, MAP_WIDTH};
use crate::death::DeathPlugin;
use crate::debugs::DebugPlugin;
use crate::enemys::EnemyPlugin;
use crate::enemys::EnemySpawnerPlugin;
use crate::systems::CameraPlugin;
use crate::systems::{AmbientPlugin, YSortPlugin};
use crate::ui::UiPlugin;
use crate::ui::life_support::LifeSupportPlugin;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

fn main() {
    App::new()
        .insert_resource(map::map_gen::generate_map(MAP_WIDTH, MAP_HEIGHT))
        .insert_resource(FlowFields::default())
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TilemapPlugin,
        ))
        .add_plugins((
            EnemySpawnerPlugin,
            ColonistSpawnerPlugin,
            map::MapRendererPlugin,
            CharacterPlugin,
            AiPlugin,
            EnemyPlugin,
            CameraPlugin,
            BuildingPlugin,
            AmbientPlugin,
            SelectionPlugin,
            CombatPlugin,
            DeathPlugin,
            UiPlugin,
            LifeSupportPlugin,
            AnimationPlugin,
        ))
        .add_plugins((ColonistAnimationPlugin, YSortPlugin))
        // Debug //
        .add_plugins(DebugPlugin)
        .run();
}
