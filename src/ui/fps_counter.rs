use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

/// Marker for the FPS counter's text entity.
#[derive(Component)]
pub struct FpsCounterText;

/// Plugin for displaying the FPS counter in the UI.
pub struct FpsCounterPlugin;
impl Plugin for FpsCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, spawn_fps_counter)
            .add_systems(Update, update_fps_counter);
    }
}

/// Spawns the FPS counter text entity in the UI.
fn spawn_fps_counter(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        Text::new("FPS: --"),
        FpsCounterText,
    ));
}

/// Updates the FPS counter text with the current FPS value.
fn update_fps_counter(diagnostics: Res<DiagnosticsStore>, mut text: Query<&mut Text, With<FpsCounterText>>) {
    let Ok(mut text) = text.single_mut() else { return; };
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d| d.smoothed()) else { return; };
    **text = format!("FPS: {:.0}", fps);
}
