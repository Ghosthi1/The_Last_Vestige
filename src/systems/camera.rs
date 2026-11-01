use crate::constants::{CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM, CAMERA_WASD_SPEED, CAMERA_ZOOM_SPEED};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

/// This plugin handles camera movement and zooming.
/// It allows the user to move the camera using WASD keys,
/// pan the camera with the middle mouse button, and zoom in and out with the mouse scroll wheel.
/// The camera's position is stored in a separate component to allow for smooth movement and snapping to grid when necessary.
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                zoom_camera,
                pan_camera,
                camera_wasd_move,
                snap_camera_to_true_position
                    .after(camera_wasd_move)
                    .after(pan_camera)
                    .after(zoom_camera),
            ),
        );
        app.add_systems(Startup, setup);
    }
}

/// This component stores the true position of the camera in world space unrounded by the current zoom level.
/// It is used to calculate the camera's position in the world and to snap the camera to the nearest pixel grid when necessary.
#[derive(Component)]
struct CameraTruePosition(Vec2);

/// This system sets up the camera and its initial position.
fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, CameraTruePosition(Vec2::ZERO)));
}

/// Zooming the camera in and out based on mouse scroll input.
/// The zoom level is clamped with constants [`CAMERA_MAX_ZOOM`] and [`CAMERA_MIN_ZOOM`] to prevent excessive zooming in or out.
/// The zoom speed is controlled by the constant [`CAMERA_ZOOM_SPEED`].
fn zoom_camera(
    scroll: Res<AccumulatedMouseScroll>,
    mut projection: Query<&mut Projection, With<Camera2d>>,
) {
    let Ok(mut proj) = projection.single_mut() else {
        return;
    };

    if let Projection::Orthographic(ref mut ortho) = *proj {
        ortho.scale = (ortho.scale * (1.0 - scroll.delta.y * CAMERA_ZOOM_SPEED)).clamp(CAMERA_MAX_ZOOM, CAMERA_MIN_ZOOM);
    }
}

/// Panning the camera based on mouse movement while the middle mouse button is pressed.
/// The camera's position is updated based on the mouse delta and the current zoom level to ensure consistent movement speed regardless of zoom.
fn pan_camera(
    button_pressed: Res<ButtonInput<MouseButton>>,
    mouse_delta: Res<AccumulatedMouseMotion>,
    mut query: Query<(&mut CameraTruePosition, &Projection), With<Camera2d>>,
) {
    if !button_pressed.pressed(MouseButton::Middle) {
        return;
    }
    let Ok((mut camera_position, projection)) = query.single_mut() else {
        return;
    };
    let movement: Vec2 = mouse_delta.delta;

    if let Projection::Orthographic(ref ortho) = *projection {
        camera_position.0.x -= movement.x * ortho.scale;
        camera_position.0.y += movement.y * ortho.scale;
    }
}

/// Moving the camera using WASD keys.
/// The camera's position is updated based on the pressed keys and the current zoom level to ensure consistent movement speed regardless of zoom.
/// The movement speed is controlled by the constant [`CAMERA_WASD_SPEED`].
fn camera_wasd_move(
    button_pressed: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut CameraTruePosition, &Projection), With<Camera2d>>,
) {
    let Ok((mut camera_position, projection)) = query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if button_pressed.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if button_pressed.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if button_pressed.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if button_pressed.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let direction = direction.normalize_or_zero();
    if direction == Vec2::ZERO {
        return;
    }

    if let Projection::Orthographic(ref ortho) = *projection {
        camera_position.0 += direction * time.delta_secs() * ortho.scale * CAMERA_WASD_SPEED;
    }
}

/// Snapping the camera's position to the nearest screen pixel grid based on the current zoom level.
/// This ensures that the camera's position aligns with the grid,
/// The snapping is done by rounding the camera's position to the nearest multiple of the current zoom level, which is determined by the orthographic projection.
fn snap_camera_to_true_position(
    mut query: Query<(&mut Transform, &CameraTruePosition, &Projection), With<Camera2d>>,
) {
    let Ok((mut transform, camera_position, projection)) = query.single_mut() else {
        return;
    };

    if let Projection::Orthographic(ref ortho) = *projection {
        transform.translation.x = (camera_position.0.x / ortho.scale).round() * ortho.scale;
        transform.translation.y = (camera_position.0.y / ortho.scale).round() * ortho.scale;
    }
}
