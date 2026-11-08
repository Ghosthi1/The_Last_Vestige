use bevy::prelude::*;

pub struct YSortPlugin;
impl Plugin for YSortPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, apply_y_sort.before(TransformSystems::Propagate));
    }
}

#[derive(Component)]
pub struct YSort;

/// Base z for y-sorted sprites, keeps them above the tilemap
const Y_SORT_BASE: f32 = 100.0;

/// How much z changes per world unit of y.
const Y_SORT_SCALE: f32 = 0.0001;

/// Converts a world space y into a draw order z, So sprites further south (Nearer camera) draw in front.
pub fn y_to_z(y: f32) -> f32 {
    Y_SORT_BASE - y * Y_SORT_SCALE
}

pub fn apply_y_sort(mut translation: Query<&mut Transform, (With<YSort>, Changed<Transform>)>){
    for mut trans in translation.iter_mut() {
        trans.translation.z = y_to_z(trans.translation.y);
    }
}
