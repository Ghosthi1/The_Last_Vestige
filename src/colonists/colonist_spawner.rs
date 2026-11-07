use std::collections::VecDeque;
use bevy::prelude::*;
use crate::animation::{load_knight_animation, KnightAnimation};
use crate::colonists::Colonist;
use crate::components::{AnimationPlayback, Attacker, GridPosition, Health, Path, Speed, Target};
use crate::constants::{COLONIST_DAMAGE, COLONIST_HEALTH, COLONIST_RANGE, COLONIST_SPEED, TILE_SIZE};
use crate::map::Map;

pub struct ColonistSpawnerPlugin;
impl Plugin for ColonistSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_colonist.after(load_knight_animation));
    }
}

fn spawn_colonist(mut commands: Commands, map: Res<Map>, knight_animation: Res<KnightAnimation>) {
    commands.spawn((
        Colonist,
        GridPosition((30,30)),
        Health::new(COLONIST_HEALTH),
        Speed(COLONIST_SPEED),
        Sprite {
            image: knight_animation.idle.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE.x)),
            texture_atlas: Some(TextureAtlas { layout: knight_animation.layout_4x2.clone(), index: 0 }),
            ..default()
        },
        Transform::from_translation(
            map.grid_to_world(30, 30).extend(1.0)
        ),
        Path(VecDeque::new()),
        Attacker::new(COLONIST_DAMAGE, COLONIST_RANGE, Timer::from_seconds(1.0, TimerMode::Repeating)),
        Target(None),
        AnimationPlayback {
            image: knight_animation.idle.clone(),
            layout: knight_animation.layout_4x2.clone(),
            frame_index: 0,
            frame_count: 4,
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        },
    ));
    commands.spawn((
        Colonist,
        GridPosition((35,35)),
        Health::new(COLONIST_HEALTH),
        Speed(COLONIST_SPEED),
        Sprite {
            image: knight_animation.idle.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE.x)),
            texture_atlas: Some(TextureAtlas { layout: knight_animation.layout_4x2.clone(), index: 0 }),
            ..default()
        },
        Transform::from_translation(
            map.grid_to_world(35, 35).extend(1.0)
        ),
        Path(VecDeque::new()),
        Attacker::new(COLONIST_DAMAGE, COLONIST_RANGE, Timer::from_seconds(1.0, TimerMode::Repeating)),
        Target(None),
        AnimationPlayback {
            image: knight_animation.idle.clone(),
            layout: knight_animation.layout_4x2.clone(),
            frame_index: 0,
            frame_count: 4,
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        },
    ));
}
