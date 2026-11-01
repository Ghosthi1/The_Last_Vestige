use bevy::prelude::*;

/// Holds the current life support values for the colony
pub struct LifeSupportPlugin;
impl Plugin for LifeSupportPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LifeSupport::default());
    }
}

/// Represents the life support resources for the colony.
#[derive(Resource)]
pub struct LifeSupport {
    /// The current oxygen level in the colony.
    pub oxygen: f32,
    /// The current food level in the colony.
    pub food: f32,
    /// The current water level in the colony.
    pub water: f32,
    /// The current energy level in the colony.
    pub energy: f32,
}

impl Default for LifeSupport {
    fn default() -> Self {
        LifeSupport {
            oxygen: 100.0,
            food: 100.0,
            water: 100.0,
            energy: 100.0,
        }
    }
}
