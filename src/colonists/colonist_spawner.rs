use std::collections::VecDeque;
use bevy::prelude::*;
use crate::colonists::Colonist;
use crate::components::{Attacker, GridPosition, Health, Path, Speed, Target};
use crate::constants::{COLONIST_DAMAGE, COLONIST_HEALTH, COLONIST_RANGE, COLONIST_SPEED, TILE_SIZE};
use crate::map::Map;

pub struct ColonistSpawnerPlugin;
impl Plugin for ColonistSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_colonist);
    }
}

fn spawn_colonist(mut commands: Commands, map: Res<Map>, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("Colonists/Knight/Knight_1.png");
    commands.spawn((
        Colonist,
        GridPosition((30,30)),
        Health::new(COLONIST_HEALTH),
        Speed(COLONIST_SPEED),
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            // offset by half map size to match the centred tilemap origin
            30.0 * TILE_SIZE + TILE_SIZE / 2.0 - map.width as f32 * TILE_SIZE / 2.0,
            30.0 * TILE_SIZE + TILE_SIZE / 2.0 - map.height as f32 * TILE_SIZE / 2.0,
            1.0
        ),
        Path(VecDeque::new()),
        Attacker::new(COLONIST_DAMAGE, COLONIST_RANGE, Timer::from_seconds(1.0, TimerMode::Repeating)),
        Target(None),
    ));
    commands.spawn((
        Colonist,
        GridPosition((35,35)),
        Health::new(COLONIST_HEALTH),
        Speed(COLONIST_SPEED),
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            // offset by half map size to match the centred tilemap origin
            35.0 * TILE_SIZE + TILE_SIZE / 2.0 - map.width as f32 * TILE_SIZE / 2.0,
            35.0 * TILE_SIZE + TILE_SIZE / 2.0 - map.height as f32 * TILE_SIZE / 2.0,
            1.0
        ),
        Path(VecDeque::new()),
        Attacker::new(COLONIST_DAMAGE, COLONIST_RANGE, Timer::from_seconds(1.0, TimerMode::Repeating)),
        Target(None),
    ));
}
