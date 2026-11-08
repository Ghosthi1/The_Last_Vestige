use bevy::prelude::*;
use rand::RngExt;
use crate::components::{Attacker, Health};
use crate::constants::{ENEMY_COUNT, ENEMY_DAMAGE, ENEMY_HEALTH, ENEMY_RANGE, ENEMY_SPAWN_EDGE_BAND, ENEMY_SPEED, TILE_SIZE};
use crate::enemys::Enemy;
use crate::map::Map;
use crate::components::movement::{GridPosition, Speed};
use crate::systems::YSort;

pub struct EnemySpawnerPlugin;
impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy);
    }
}

/// Scatters `ENEMY_COUNT` enemies across random passable tiles.
/// TEMPORARY: for capturing horde screenshots/video for the README — replace with
/// real wave/spawn-point logic once that design exists.
fn spawn_enemy(mut commands: Commands, map: Res<Map>, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("enemeys/Spiders/Grunt.png");
    let mut rng = rand::rng();
    let mut spawned = 0;

    while spawned < ENEMY_COUNT {
        let band = ENEMY_SPAWN_EDGE_BAND.min(map.width / 2).min(map.height / 2);
        let (x, y) = match rng.random_range(0..4) {
            0 => (rng.random_range(0..map.width), rng.random_range(0..band)), // top
            1 => (rng.random_range(0..map.width), rng.random_range(map.height - band..map.height)), // bottom
            2 => (rng.random_range(0..band), rng.random_range(0..map.height)), // left
            _ => (rng.random_range(map.width - band..map.width), rng.random_range(0..map.height)), // right
        };
        if !map.get(x, y).is_passable() {
            continue;
        }

        commands.spawn((
            Enemy,
            GridPosition((x, y)),
            Health::new(ENEMY_HEALTH),
            Speed(ENEMY_SPEED),
            Sprite {
                image: texture.clone(),
                custom_size: Some(Vec2::splat(TILE_SIZE.x)),
                ..default()
            },
            Transform::from_translation(
                map.grid_to_world(x, y).extend(0.0)
            ),
            Attacker::new(ENEMY_DAMAGE, ENEMY_RANGE, Timer::from_seconds(1.0, TimerMode::Repeating)),
            YSort,
        ));

        spawned += 1;
    }
}
