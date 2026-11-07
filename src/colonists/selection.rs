use bevy::prelude::*;
use bevy::math::Rect;
use bevy::window::PrimaryWindow;
use crate::colonists::Colonist;
use crate::components::Selected;
use crate::constants::TILE_SIZE;

#[derive(Resource,Default)]
pub struct DragSelection{
    drag_start: Option<Vec2>,
    is_dragging: bool,
}

pub struct SelectionPlugin;
impl Plugin for SelectionPlugin{
    fn build(&self, app: &mut App) {
        app.init_resource::<DragSelection>();
        app.add_systems(Update, (dragselection, draw_selection_indicator));
    }
}

pub fn dragselection (mouse: Res<ButtonInput<MouseButton>>, window_pos: Query<&Window, With<PrimaryWindow>>, pos: Query<(&Camera, &GlobalTransform)>,
                       mut state: ResMut<DragSelection>, colonist: Query<(Entity, &Transform), With<Colonist>>, mut commands: Commands, mut gizmos: Gizmos)
{
    let threshold = 5.0;
    if mouse.just_pressed(MouseButton::Left){
        if let Some(drag_start) = window_pos.single().unwrap().cursor_position(){
            state.drag_start = Some(drag_start);
            state.is_dragging = false;
        }else { return; }

    }
    if mouse.pressed(MouseButton::Left) {
        if let Some(current_pos) = window_pos.single().unwrap().cursor_position() {
            let distance = state.drag_start.unwrap().distance(current_pos);
            if distance > threshold {
                state.is_dragging = true;
            }
            if state.is_dragging {
                let (camera, camera_transform) = pos.single().unwrap();
                let start_result = camera.viewport_to_world_2d(camera_transform, state.drag_start.unwrap());
                let end_result = camera.viewport_to_world_2d(camera_transform, current_pos);
                if let (Ok(start_world), Ok(end_world)) = (start_result, end_result) {
                    let center = (start_world + end_world) / 2.0;
                    let size = (end_world - start_world).abs();
                    gizmos.rect_2d(Isometry2d::from_translation(center), size, Color::WHITE);
                }
            }
        }
    }

    if mouse.just_released(MouseButton::Left){
        let saved_start = state.drag_start;
        let (camera, camera_transform) = pos.single().unwrap();

        if !state.is_dragging{
            if let Some(current_pos) = window_pos.single().unwrap().cursor_position(){
                if let Ok(current_pos) = camera.viewport_to_world_2d(camera_transform, current_pos){
                    let mut minimum_distance = f32::MAX;
                    let mut closest_entity: Option<Entity> = None;
                    for (entity, transform) in colonist.iter(){
                        let dist = transform.translation.xy().distance(current_pos);
                        if dist < minimum_distance {
                            minimum_distance = dist;
                            closest_entity = Some(entity);
                        }
                    }
                    for (e, _) in colonist.iter(){
                        commands.entity(e).remove::<Selected>();
                    }

                    if let Some(winner) = closest_entity && minimum_distance < TILE_SIZE.x * 0.6 {
                        commands.entity(winner).insert(Selected);
                    }
                }
            }
        }
        else {
            if let Ok(saved_start) = camera.viewport_to_world_2d(camera_transform, saved_start.unwrap()){
                if let Some(current_pos) = window_pos.single().unwrap().cursor_position(){
                    if let Ok(current_pos) = camera.viewport_to_world_2d(camera_transform, current_pos){
                        let rect = Rect::from_corners(saved_start, current_pos);

                        for (e, t) in colonist.iter(){
                            commands.entity(e).remove::<Selected>();
                            if rect.contains(t.translation.xy()) {
                                commands.entity(e).insert(Selected);
                            }
                        }
                    }
                }
            }
        }
        state.is_dragging = false;
        state.drag_start = None;
    }
}

/// Draws a circle around the selected targets
pub fn draw_selection_indicator(selected: Query<&Transform, With<Selected>>, mut gizmos: Gizmos){
    for selected in selected.iter(){
        gizmos.circle_2d( Isometry2d::from_translation(selected.translation.xy()), TILE_SIZE.x * 0.5, Color::WHITE);
    }
}
