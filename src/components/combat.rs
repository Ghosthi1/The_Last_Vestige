use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    current: f32,
    max: f32,
}
impl Health {
    /// Assigns the max health of the entity
    pub fn new(max: f32) -> Self {
        debug_assert!(max > 0.0 );
        Self{current: max, max}
    }

    /// Updates the current health depending on the health received, "-" for damage and + for healing
    pub fn change_health(&mut self, health: f32) {
        self.current += health;
        if self.current < 0.0 {
            self.current = 0.0;
            // TODO
            // fn destroy_entity, cleans up entities with the destroyed tag, so change the entities tag here
        }
        if self.current > self.max {
            self.current = self.max;
        }
    }

    /// checks if the entity is dead
    pub fn is_dead(&self) -> bool{
        self.current <= 0.0
    }
}

#[derive(Component)]
pub struct Attacker{
    pub damage: f32,
    pub range: f32,
    pub cooldown: Timer
}
impl Attacker {
    pub fn new(damage: f32, range:f32, cooldown: Timer) -> Self {
        Self{damage, range, cooldown}
    }
}

#[derive(Component)]
pub struct Target(pub Option<Entity>);