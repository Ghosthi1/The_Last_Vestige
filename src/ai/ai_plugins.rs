use bevy::prelude::*;

use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Query, Res};
use crate::map::Map;
use crate::ai::{FlowFields};
use crate::character::{Colonist, GridPosition};

pub struct AiPlugin;
impl Plugin for AiPlugin {
    fn build(&self, app: &mut App ) {
        app.add_systems(Update, rebuild_colonist_flow_field);
    }
}

pub fn rebuild_colonist_flow_field(mut flow_fields: ResMut<FlowFields>, map: Res<Map>, colonist_moved: Query<&GridPosition, (With<Colonist>, Changed<GridPosition>)>,
                               colonist_pos: Query<&GridPosition, With<Colonist>>, mut positions: Local<Vec<(u32, u32)>>) {
    if colonist_moved.is_empty() { return } // no colonist moved

    positions.clear();

    for grid_pos in colonist_pos.iter() {
        positions.push(grid_pos.0);
    }
    flow_fields.colonists.build_flow_fields(&map, &positions)

}