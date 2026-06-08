use bevy::prelude::*;
use crate::colonists::Colonist;
use crate::components::{Attacker, Health, Target};
use crate::enemys::Enemy;

pub struct CombatPlugin;
impl Plugin for CombatPlugin {

    fn build(&self, app: &mut App) {
        app.add_systems(Update, (enemy_attack, colonist_attack));
    }
}

pub fn enemy_attack(mut enemy_query: Query<(&mut Attacker, &Transform), (With<Enemy>, Without<Colonist>)>, mut colonists:Query<(Entity,&Transform, &mut Health), (With<Colonist>, Without<Enemy>)>,
                    time: Res<Time>)
{
    let mut colonist_snapshot: Vec<(Entity, Vec2)> = Vec::new();
    for (entity, transform, _health) in colonists.iter() {
        colonist_snapshot.push((entity, transform.translation.truncate()));
    }

    for (mut attacker, transform) in enemy_query.iter_mut() {
        attacker.cooldown.tick(time.delta());
        let mut closest = (f32::MAX, None::<Entity>) ;

        let enemy_world_pos = transform.translation.truncate();

        for (entity, colonist_pos) in colonist_snapshot.iter() {
            let distance = enemy_world_pos.distance_squared(*colonist_pos);
            if distance < closest.0 {
                closest.0 = distance;
                closest.1 = Some(*entity);
            }
        }
        if closest.0 <= attacker.range * attacker.range && attacker.cooldown.just_finished(){
            if let Some(target) = closest.1 {
                if let Ok((_,_, mut health)) = colonists.get_mut(target) {
                    health.change_health(-attacker.damage);
                }
            }
        }
    }
}

pub fn colonist_attack(mut colonists:Query<(&mut Attacker, &Transform, &Target), (With<Colonist>, Without<Enemy>)>, mut enemy_query: Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Colonist>)>,
                       time: Res<Time>)
{
    let mut enemy_snapshot: Vec<(Entity, Vec2)> = Vec::new();
    for (entity, transform, _health) in enemy_query.iter() {
        enemy_snapshot.push((entity, transform.translation.truncate()));
    }

    for (mut attacker, transform,target) in colonists.iter_mut() {
        let colonist_world_pos = transform.translation.truncate();
        attacker.cooldown.tick(time.delta());

        match target.0 {
            Some(entity) => {
                if let Some((_, enemy_pos)) = enemy_snapshot.iter().find(|(e, _)| *e == entity) {
                    let distance = colonist_world_pos.distance_squared(*enemy_pos);
                    if distance <= attacker.range * attacker.range && attacker.cooldown.just_finished() {
                        if let Ok((_, _, mut health)) = enemy_query.get_mut(entity) {
                            health.change_health(-attacker.damage);
                        }
                    }
                }
            },
            None => {
                let mut closest = (f32::MAX, None::<Entity>) ;

                for (entity, colonist_pos) in enemy_snapshot.iter() {
                    let distance = colonist_world_pos.distance_squared(*colonist_pos);
                    if distance < closest.0 {
                        closest.0 = distance;
                        closest.1 = Some(*entity);
                    }
                }
                if closest.0 <= attacker.range * attacker.range && attacker.cooldown.just_finished(){
                    if let Some(target) = closest.1 {
                        if let Ok((_,_, mut health)) = enemy_query.get_mut(target) {
                            health.change_health(-attacker.damage);
                        }
                    }
                }
            }
        }
    }
}
