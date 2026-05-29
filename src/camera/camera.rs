use bevy::prelude::*;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (zoom_camera, pan_camera));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn zoom_camera(scroll:  Res<AccumulatedMouseScroll>, mut projection: Query<&mut Projection, With<Camera2d>>) {
    let Ok(mut proj) = projection.single_mut() else { return };
    let sensitivity = 0.1;

    if let Projection::Orthographic(ref mut ortho) = *proj {
        ortho.scale = (ortho.scale *  (1.0 - scroll.delta.y * sensitivity)).clamp(0.3, 3.0);
    }
}

fn pan_camera(button_pressed: Res<ButtonInput<MouseButton>>, mouse_delta: Res<AccumulatedMouseMotion>, mut query: Query<(&mut Transform, &Projection), With<Camera2d>> ) {
    if !button_pressed.pressed(MouseButton::Middle){ return; }
    let Ok((mut transform, projection)) = query.single_mut() else { return };
    let movement: Vec2 = mouse_delta.delta;

    if let Projection::Orthographic(ref ortho) = *projection {
        transform.translation.x -= movement.x * ortho.scale;
        transform.translation.y += movement.y * ortho.scale;
    }
}