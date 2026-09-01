use bevy::asset::Handle;
use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use bevy::time::Timer;

/// Represents which animation the colonist is currently playing
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColonistAnimationState{
    Idle,
    Walk,
    Attack,
    Carry,
    Work,
    Sleep
}

/// Represents which animation the Enemy is currently playing
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyAnimationState{
    Idle,
    Walk,
    Attack,
    Death,
}

/// The state of animation for an entity
#[derive(Component)]
pub struct AnimationPlayback{
    /// The current active sheet
    pub image: Handle<Image>,
    /// The current layout
    pub layout: Handle<TextureAtlasLayout>,
    /// Which sub frame is showing now
    pub frame_index: usize,
    /// How many frames in the current cycle
    pub frame_count: usize,
    pub timer: Timer,
}