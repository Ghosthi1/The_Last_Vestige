use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use crate::constants::*;
use crate::enemys::Enemy;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugSettings::default());
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());

        app.add_systems(Update, controls);
        app.add_systems(Update, grid_lines);
        app.add_systems(Update, ui_elements);
        app.add_systems(Startup, spawn_enemy_counter)
            .add_systems(Update, update_enemy_counter);
        app.add_systems(Startup, spawn_fps_counter)
            .add_systems(Update, update_fps_counter);
    }
}

#[derive(Resource, Default)]
pub struct DebugSettings {
    pub grid_lines: bool,
    pub ui_elements: bool,
}

#[derive(Component)]
pub struct DebugUi;

fn controls(keys: Res<ButtonInput<KeyCode>>, mut debug_settings: ResMut<DebugSettings>) {
    if keys.just_pressed(KeyCode::F1) {
        println!("Running Debug grid_lines");
        debug_settings.grid_lines = !debug_settings.grid_lines;
    }
    if keys.just_pressed(KeyCode::F2) {
        println!("Running Debug ui_elements");
        debug_settings.ui_elements = !debug_settings.ui_elements;
    }
}


/// Creates Lines so the grid is visible
fn grid_lines(mut gizmos: Gizmos, debug_settings: Res<DebugSettings>) {
    if !debug_settings.grid_lines {
        return;
    }
    for i in 0..=MAP_WIDTH{
        let x = -(MAP_WIDTH as f32 * TILE_SIZE / 2.0) + i as f32 * TILE_SIZE;
        gizmos.line_2d( Vec2::new(x, -(MAP_HEIGHT as f32 * TILE_SIZE / 2.0)),  Vec2::new(x, MAP_HEIGHT as f32 * TILE_SIZE / 2.0), Color::WHITE)
    }
    for j in 0..=MAP_HEIGHT{
        let y = -(MAP_HEIGHT as f32 * TILE_SIZE / 2.0) + j as f32 * TILE_SIZE;
        gizmos.line_2d(  Vec2::new(-(MAP_WIDTH as f32 * TILE_SIZE / 2.0), y), Vec2::new(MAP_WIDTH as f32 * TILE_SIZE / 2.0, y), Color::WHITE)
    }
}

/// Creates UI elements for debugging (like FPS counter, enemy counter, etc.)
fn ui_elements(debug_settings: Res<DebugSettings>, mut debug_ui: Query<&mut Visibility , With<DebugUi>>) {
    if !debug_settings.is_changed(){
        return;
    }

    for mut visibility in debug_ui.iter_mut() {
        *visibility = if debug_settings.ui_elements {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[derive(Component)]
pub struct EnemyCounterText;

/// Spawns a UI text element to display the current number of enemies in the game.
fn spawn_enemy_counter(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        Text::new("Enemies: --"),
        EnemyCounterText,
        Visibility::Hidden,
        DebugUi
    ));
}

/// Updates the enemy counter text to reflect the current number of enemies in the game.
fn update_enemy_counter(
    enemies: Query<(), With<Enemy>>,
    mut text: Query<&mut Text, With<EnemyCounterText>>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    **text = format!("Enemies: {}", enemies.iter().count());
}

#[derive(Component)]
pub struct FpsCounterText;

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
        Visibility::Hidden,
        DebugUi
    ));
}

/// Updates the FPS counter text with the current FPS value.
fn update_fps_counter(diagnostics: Res<DiagnosticsStore>, mut text: Query<&mut Text, With<FpsCounterText>>) {
    let Ok(mut text) = text.single_mut() else { return; };
    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|d| d.smoothed()) else { return; };
    **text = format!("FPS: {:.0}", fps);
}




