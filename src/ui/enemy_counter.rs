use bevy::prelude::*;
use crate::enemys::Enemy;

/// Marker for the enemy counter's text entity.
#[derive(Component)]
pub struct EnemyCounterText;

pub struct EnemyCounterPlugin;
impl Plugin for EnemyCounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy_counter)
            .add_systems(Update, update_enemy_counter);
    }
}

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
    ));
}

fn update_enemy_counter(enemies: Query<(), With<Enemy>>, mut text: Query<&mut Text, With<EnemyCounterText>>) {
    let Ok(mut text) = text.single_mut() else { return; };
    **text = format!("Enemies: {}", enemies.iter().count());
}
