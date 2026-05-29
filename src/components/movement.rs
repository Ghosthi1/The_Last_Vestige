use bevy::prelude::*;
use std::collections::VecDeque;

/// Current tile coordinates
#[derive(Component)]
pub struct GridPosition(pub(u32, u32));

/// Path to the target
#[derive(Component)]
pub struct Path (pub VecDeque<(u32,u32)>);

/// The movement speed in tiles per second
#[derive(Component)]
pub struct Speed (pub f32);