use crate::{
    character::Character,
    enemy::Enemy,
    map::{GameMap, HexCoord, MapEvent},
    party::{Group, GroupCommandsExt, GroupMember},
    State,
};
use bevy::prelude::*;
use core::{ops::Range, time::Duration};
use rand::Rng;
use smallvec::SmallVec;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (
                initiate_combat,
                combat_round,
                despawn_no_health.after(combat_round),
                finish_combat.after(despawn_no_health),
            )
                .in_set(OnUpdate(State::Running)),
        );
    }
}

#[derive(Component)]
pub struct Combat {
    position: HexCoord,
    timer: Timer,
    friends: SmallVec<[Entity; 8]>,
    foes: SmallVec<[Entity; 8]>,
}

#[derive(Component, Default)]
pub struct Health(pub u16);

#[derive(Component, Default)]
pub struct Attack(pub Range<u16>);

pub fn initiate_combat(
    mut commands: Commands,
    mut map_events: EventReader<MapEvent>,
    map_query: Query<&GameMap>,
    friend_query: Query<&Group>,
    character_query: Query<Entity, With<Character>>,
    foe_query: Query<Entity, With<Enemy>>,
) {
    let Ok(map) = map_query.get_single() else { return };
    for event in &mut map_events {
        let MapEvent::PresenceMoved { position, .. } = event else { continue };
        let friends: SmallVec<[Entity; 8]> = friend_query
            .iter_many(map.presence(*position))
            .flat_map(|group| character_query.iter_many(&group.members))
            .collect();
        let foes: SmallVec<[Entity; 8]> = foe_query.iter_many(map.presence(*position)).collect();
        if !friends.is_empty() && !foes.is_empty() {
            commands.spawn(Combat {
                position: *position,
                timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
                friends,
                foes,
            });
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn combat_round(
    mut combat_query: Query<&mut Combat>,
    time: Res<Time>,
    mut friend_query: Query<(&Attack, &mut Health), (With<Character>, Without<Enemy>)>,
    mut foe_query: Query<(&Attack, &mut Health), (With<Enemy>, Without<Character>)>,
) {
    let mut rng = rand::thread_rng();
    for mut combat in &mut combat_query {
        combat.timer.tick(time.delta());
        if !combat.timer.just_finished() {
            continue;
        }

        info!("Combat at {}", combat.position);

        for (attack, _health) in friend_query.iter_many(&combat.friends) {
            let mut foe_iter = foe_query.iter_many_mut(&combat.foes);
            while let Some((_, mut health)) = foe_iter.fetch_next() {
                if health.0 == 0 {
                    continue;
                }
                health.0 = health.0.saturating_sub(rng.gen_range(attack.0.clone()));
                break;
            }
        }

        for (attack, _health) in foe_query.iter_many(&combat.foes) {
            let mut friend_iter = friend_query.iter_many_mut(&combat.friends);
            while let Some((_, mut health)) = friend_iter.fetch_next() {
                if health.0 == 0 {
                    continue;
                }
                health.0 = health.0.saturating_sub(rng.gen_range(attack.0.clone()));
                break;
            }
        }
    }
}

pub fn despawn_no_health(
    mut commands: Commands,
    health_query: Query<(Entity, &Health, Option<&GroupMember>)>,
) {
    for (entity, health, maybe_member) in &health_query {
        if health.0 == 0 {
            info!("{:?} is dead", entity);
            if let Some(member) = maybe_member {
                commands.entity(member.group).remove_members(&[entity]);
            }
            commands.entity(entity).despawn();
        }
    }
}

pub fn finish_combat(
    mut commands: Commands,
    combat_query: Query<(Entity, &Combat)>,
    friend_query: Query<Entity, (With<Character>, Without<Enemy>)>,
    foe_query: Query<Entity, (With<Enemy>, Without<Character>)>,
) {
    for (entity, combat) in &combat_query {
        let no_friends = friend_query.iter_many(&combat.friends).next().is_none();
        let no_foes = foe_query.iter_many(&combat.foes).next().is_none();
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
