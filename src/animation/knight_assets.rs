use bevy::prelude::*;
use crate::animation::playback::advance_animation;

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_knight_animation);
        app.add_systems(Update, advance_animation);
    }
}

/// Holds the knight animation assets
#[derive(Resource)]
pub struct KnightAnimation {
    pub idle: Handle<Image>,
    pub walk: Handle<Image>,
    pub attack: Handle<Image>,
    pub carry: Handle<Image>,
    pub work: Handle<Image>,
    pub sleep: Handle<Image>,
    ///  The grid layout of the sprite sheet, (idle, walk, attack, carry, work)
    pub layout_4x2: Handle<TextureAtlasLayout>,
    /// The grid layout of the sprite sheet, (sleep)
    pub layout_2x1: Handle<TextureAtlasLayout>,
}

pub fn load_knight_animation(mut commands: Commands, asset_server: Res<AssetServer>,
                             mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>){
    let knight_animation = KnightAnimation {
        idle: asset_server.load("Colonists/Knight/space_knight_idle_sheet_4x2_128px.png"),
        walk: asset_server.load("Colonists/Knight/space_knight_walk_sheet_4x2_128px.png"),
        attack: asset_server.load("Colonists/Knight/space_knight_attack_sheet_4x2_128px.png"),
        carry: asset_server.load("Colonists/Knight/space_knight_carry_sheet_4x2_128px.png"),
        work: asset_server.load("Colonists/Knight/space_knight_work_sheet_4x2_128px.png"),
        sleep: asset_server.load("Colonists/Knight/space_knight_sleep_sheet_2x1_128px.png"),
        layout_4x2: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 4, 2, None, None)),
        layout_2x1: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::new(128, 128), 2, 1, None, None)),
    };
    commands.insert_resource(knight_animation);
}
