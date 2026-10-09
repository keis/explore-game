use bevy::prelude::*;

#[derive(Message)]
pub enum CombatEvent {
    Initiate(Entity),
    FriendDamage(Entity, u16),
    EnemyDamage(Entity, u16),
}
