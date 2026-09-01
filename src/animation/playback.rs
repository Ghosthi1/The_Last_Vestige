use bevy::prelude::*;
use crate::components::AnimationPlayback;

pub fn advance_animation(time: Res<Time>, mut query: Query<(&mut AnimationPlayback, &mut Sprite)>) {
    for (mut playback, mut sprite) in &mut query {
        playback.timer.tick(time.delta());
        if sprite.image != playback.image {
            sprite.image = playback.image.clone();
        }
        if playback.timer.just_finished() {
            playback.frame_index = (playback.frame_index + 1) % playback.frame_count;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = playback.frame_index;
            }
        }
    }
}