use bevy::prelude::*;

/// Responsible for playing ambient sounds in the game.
pub struct AmbientPlugin;
impl Plugin for AmbientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ambient);
    }
}

/// Plays ambient sounds when the game starts. On a loop, it plays the ambient spaceship sound.
fn ambient(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ambient_spaceship = asset_server.load("Sound/Background/ambient_spaceship.ogg");
    commands.spawn((AudioPlayer::new(ambient_spaceship), PlaybackSettings::LOOP));
}
