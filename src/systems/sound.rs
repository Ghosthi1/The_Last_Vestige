use bevy::prelude::*;

pub struct AmbientPlugin;
impl Plugin for AmbientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ambient);
    }
}

fn ambient (mut commands: Commands, asset_server: Res<AssetServer>) {
    let ambient_spaceship = asset_server.load( "Sound/Background/ambient_spaceship.ogg");
    commands.spawn((AudioPlayer::new(ambient_spaceship), PlaybackSettings::LOOP));

}