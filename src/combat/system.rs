use super::{bundle::*, component::*, event::*};
use crate::{
    actor::{Character, Corpse, Enemy, Group, GroupCommandsExt, GroupMember},
    floating_text::{FloatingTextAlignment, FloatingTextPrototype, FloatingTextSource},
    map::{MapCommandsExt, MapEvent, PresenceLayer},
};
use bevy::prelude::*;
use rand::Rng;

pub fn combat_log(mut combat_events: EventReader<CombatEvent>, combat_query: Query<&Combat>) {
    for event in combat_events.read() {
        match event {
            CombatEvent::Initiate(entity) => {
                let Ok(combat) = combat_query.get(*entity) else {
                    continue;
                };
                info!("Combat initiated at {}!", combat.position)
            }
            CombatEvent::FriendDamage(entity, damage) => {
                let Ok(combat) = combat_query.get(*entity) else {
                    continue;
                };
                info!(
                    "Damage to friendly in combat at {} - {}",
                    combat.position, damage
                )
            }
            CombatEvent::EnemyDamage(entity, damage) => {
                let Ok(combat) = combat_query.get(*entity) else {
                    continue;
                };
                info!(
                    "Damage to enemy in combat at {} - {}",
                    combat.position, damage
                )
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initiate_combat(
    mut commands: Commands,
    mut map_events: EventReader<MapEvent>,
    mut combat_events: EventWriter<CombatEvent>,
    mut combat_params: CombatParams,
    map_query: Query<&PresenceLayer>,
    friend_query: Query<&Group>,
    character_query: Query<Entity, With<Character>>,
    foe_query: Query<Entity, With<Enemy>>,
) {
    let Ok(presence_layer) = map_query.get_single() else {
        return;
    };
    for event in map_events.read() {
        let MapEvent::PresenceMoved { position, .. } = event else {
            continue;
        };
        let friends: Vec<_> = friend_query
            .iter_many(presence_layer.presence(*position))
            .flat_map(|group| character_query.iter_many(&group.members))
            .collect();
        let foes: Vec<_> = foe_query
            .iter_many(presence_layer.presence(*position))
            .collect();
        let initiative_order = friends.iter().chain(foes.iter()).cloned().collect();
        if !friends.is_empty() && !foes.is_empty() {
            let entity = commands
                .spawn(CombatBundle::new(
                    &mut combat_params,
                    *position,
                    initiative_order,
                ))
                .id();
            combat_events.send(CombatEvent::Initiate(entity));
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn combat_round(
    mut combat_query: Query<(Entity, &mut Combat)>,
    mut combat_events: EventWriter<CombatEvent>,
    attacker_query: Query<(&Attack, Option<&Enemy>)>,
    mut target_query: Query<(&mut Health, Option<&Enemy>)>,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut combat) in &mut combat_query {
        info!("Combat at {}", combat.position);

        let attacker_entity = combat.initiative_order[combat.initiative];
        combat.initiative = (combat.initiative + 1) % combat.initiative_order.len();

        let Ok((attack, maybe_attacker_enemy)) = attacker_query.get(attacker_entity) else {
            continue;
        };
        let mut target_iter = target_query.iter_many_mut(&combat.initiative_order);
        while let Some((mut health, maybe_target_enemy)) = target_iter.fetch_next() {
            if health.0 == 0 || maybe_attacker_enemy.is_some() == maybe_target_enemy.is_some() {
                continue;
            }
            let damage = rng.gen_range(attack.range()).min(health.0);
            health.0 -= damage;
            combat_events.send(if maybe_target_enemy.is_some() {
                CombatEvent::EnemyDamage(entity, damage)
            } else {
                CombatEvent::FriendDamage(entity, damage)
            });
            break;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn make_corpses(
    mut commands: Commands,
    map_query: Query<Entity, With<PresenceLayer>>,
    health_query: Query<(Entity, &Health, Option<&GroupMember>, Option<&Enemy>), Without<Corpse>>,
) {
    let Ok(map_entity) = map_query.get_single() else {
        return;
    };
    for (entity, health, maybe_member, maybe_enemy) in &health_query {
        if health.0 == 0 {
            info!("{:?} is dead", entity);
            if let Some(group) = maybe_member.and_then(|member| member.group) {
                commands.entity(group).remove_members(&[entity]);
            }
            if maybe_enemy.is_some() {
                commands.entity(map_entity).despawn_presence(entity);
            } else {
                commands.entity(entity).insert(Corpse);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn finish_combat(
    mut commands: Commands,
    combat_query: Query<(Entity, &Combat)>,
    friend_query: Query<Entity, (With<Character>, Without<Corpse>, Without<Enemy>)>,
    foe_query: Query<Entity, (With<Enemy>, Without<Character>)>,
) {
    for (entity, combat) in &combat_query {
        let no_friends = friend_query
            .iter_many(&combat.initiative_order)
            .next()
            .is_none();
        let no_foes = foe_query
            .iter_many(&combat.initiative_order)
            .next()
            .is_none();
        if no_friends && no_foes {
            info!("Combat at {} leaves no one alive", combat.position);
        } else if no_friends {
            info!("Suffers defeat in combat at {}", combat.position);
        } else if no_foes {
            info!("Victorious in combat at {}", combat.position);
        } else {
            continue;
        }

        commands.entity(entity).despawn();
    }
}

pub fn spawn_damage_text(
    mut combat_events: EventReader<CombatEvent>,
    mut combat_query: Query<&mut FloatingTextSource, With<Combat>>,
) {
    for event in combat_events.read() {
        match event {
            CombatEvent::FriendDamage(entity, damage) => {
                let Ok(mut floating_text_source) = combat_query.get_mut(*entity) else {
                    continue;
                };
                floating_text_source.add(FloatingTextPrototype {
                    value: damage.to_string(),
                    alignment: FloatingTextAlignment::Left,
                    color: Color::RED,
                });
            }
            CombatEvent::EnemyDamage(entity, damage) => {
                let Ok(mut floating_text_source) = combat_query.get_mut(*entity) else {
                    continue;
                };
                floating_text_source.add(FloatingTextPrototype {
                    value: damage.to_string(),
                    alignment: FloatingTextAlignment::Right,
                    color: Color::YELLOW,
                });
            }
            _ => {}
        }
    }
}
