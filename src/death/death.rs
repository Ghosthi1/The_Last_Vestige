use bevy::prelude::*;
use crate::colonists::Colonist;
use crate::components::{Dead, Health};
use crate::enemys::Enemy;
use crate::combat::{colonist_attack, enemy_attack};

pub struct DeathPlugin;
impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tag_dead.after(colonist_attack).after(enemy_attack),
                                dead_enemies_handler.after(tag_dead), dead_colonists_handler.after(tag_dead)));
    }
}

/// Tags entities whose health is at zero with dead
pub fn tag_dead(health_query:Query<(Entity, &Health), (Without<Dead>, Changed<Health>)>, mut commands: Commands) {
    for (entity, health) in health_query.iter() {
        if health.is_dead() {commands.entity(entity).insert(Dead);}
    }
}

/// Handles enemys marked with the dead tag
/// Despawning, Animation, Effects
pub fn dead_enemies_handler(dead_enemys:Query<Entity, (With<Dead>, With<Enemy>)>, mut commands: Commands){
    for entity in dead_enemys.iter() {
        commands.entity(entity).despawn();
    }
}

///  Handles colonists marked with the dead tag
/// Despawning, Animation, Effects, Inventory management, Relationships,
pub fn dead_colonists_handler(dead_colonists:Query<Entity, (With<Dead>, With<Colonist>)>, mut commands: Commands){
    for entity in dead_colonists.iter() {
        commands.entity(entity).despawn();
    }
}