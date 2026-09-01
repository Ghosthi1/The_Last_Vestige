use std::time::Duration;
use bevy::prelude::*;
use crate::animation::{advance_animation, KnightAnimation};
use crate::colonists::Colonist;
use crate::components::{AnimationPlayback, Attacker, Path};
use crate::enemys::Enemy;

pub struct ColonistAnimationPlugin;
impl Plugin for ColonistAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, decide_colonist_animation.before(advance_animation));
    }
}

pub fn decide_colonist_animation(mut colonists: Query<(&Attacker, &Transform, &Path, &mut AnimationPlayback), With<Colonist>>
                                 , enemies: Query<&Transform, With<Enemy>>, knight_animation: Res<KnightAnimation>) {
    let mut enemy_snapshot: Vec<Vec2> = Vec::new();
    for transform in enemies.iter() {
        enemy_snapshot.push(transform.translation.truncate());
    }

    for (attacker, transform, path, mut animation) in colonists.iter_mut() {
        let colonist_pos = transform.translation.truncate();
        let in_range = enemy_snapshot.iter().any(|pos| colonist_pos.distance_squared(*pos) <= attacker.range * attacker.range);

        let desired_image = if in_range {
            knight_animation.attack.clone()
        } else if !path.0.is_empty() {
            knight_animation.walk.clone()
        } else {
            knight_animation.idle.clone()
        };

        let desired_duration = if in_range {
            0.3
        } else if !path.0.is_empty() {
            0.2
        } else {
            0.2
        };

        if animation.image != desired_image {
            animation.image = desired_image;
            animation.frame_index = 0;
            animation.timer.reset();
            animation.timer.set_duration(Duration::from_secs_f32(desired_duration));
        }
    }
}