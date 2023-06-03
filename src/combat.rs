use crate::{
    action::{follow_path, ActionSet},
    assets::MainAssets,
    character::Character,
    enemy::Enemy,
    map::{HexCoord, MapEvent, PresenceLayer},
    party::{Group, GroupCommandsExt, GroupMember},
    State,
};
use bevy::prelude::*;
use bevy_sprite3d::{Sprite3d, Sprite3dBundle, Sprite3dParams};
use core::{ops::Range, time::Duration};
use rand::Rng;
use smallvec::SmallVec;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CombatEvent>()
            .add_system(
                initiate_combat
                    .before(follow_path)
                    .in_base_set(ActionSet::PostApply)
                    .run_if(on_event::<MapEvent>()),
            )
            .add_systems(
                (
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

#[derive(Bundle)]
pub struct CombatBundle {
    pub combat: Combat,
    pub sprite3d: Sprite3dBundle,
}

pub type CombatParams<'w, 's> = (Res<'w, MainAssets>, Sprite3dParams<'w, 's>);

impl CombatBundle {
    pub fn new(
        (main_assets, sprite_params): &mut CombatParams,
        position: HexCoord,
        friends: SmallVec<[Entity; 8]>,
        foes: SmallVec<[Entity; 8]>,
    ) -> Self {
        Self {
            combat: Combat {
                position,
                timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
                friends,
                foes,
            },
            sprite3d: Sprite3d {
                image: main_assets.swords_emblem_icon.clone(),
                pixels_per_metre: 400.0,
                transform: Transform::from_translation(
                    Vec3::from(position) + Vec3::new(0.0, 1.0, 0.0),
                ),
                ..default()
            }
            .bundle(sprite_params),
        }
    }
}

#[derive(Component, Default)]
pub struct Health(pub u16);

#[derive(Component, Default)]
pub struct Attack(pub Range<u16>);

pub enum CombatEvent {
    Initiate,
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
    let Ok(presence_layer) = map_query.get_single() else { return };
    for event in &mut map_events {
        let MapEvent::PresenceMoved { position, .. } = event else { continue };
        let friends: SmallVec<[Entity; 8]> = friend_query
            .iter_many(presence_layer.presence(*position))
            .flat_map(|group| character_query.iter_many(&group.members))
            .collect();
        let foes: SmallVec<[Entity; 8]> = foe_query
            .iter_many(presence_layer.presence(*position))
            .collect();
        if !friends.is_empty() && !foes.is_empty() {
            commands.spawn(CombatBundle::new(
                &mut combat_params,
                *position,
                friends,
                foes,
            ));
            combat_events.send(CombatEvent::Initiate);
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
