use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud_root);
    }
}

#[derive(Resource)]
pub struct LifeSupport {
    oxygen:f32,
    food:f32,
    water:f32,
    energy:f32,
}

impl Default for LifeSupport {
    fn default() -> Self{
        LifeSupport {oxygen: 100.0, food: 100.0, water: 100.0, energy: 100.0}
    }
}

pub fn spawn_hud_root(mut commands: Commands) {
    commands.spawn((Node{
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
    ..Default::default()},));
}


