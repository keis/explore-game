use bevy::prelude::*;

#[derive(Event)]
pub enum CombatEvent {
    Initiate(Entity),
    FriendDamage(Entity, u16),
    EnemyDamage(Entity, u16),
}
