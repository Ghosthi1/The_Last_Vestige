use bevy::prelude::*;

pub struct LifeSupportPlugin;
impl Plugin for LifeSupportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LifeSupport::default());
    }
}

#[derive(Resource)]
pub struct LifeSupport {
    pub oxygen:f32,
    pub food:f32,
    pub water:f32,
    pub energy:f32,
}

impl Default for LifeSupport {
    fn default() -> Self{
        LifeSupport {oxygen: 100.0, food: 100.0, water: 100.0, energy: 100.0}
    }
}
