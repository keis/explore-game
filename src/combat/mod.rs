use crate::{
    action::ActionSet,
    actor::{Character, Enemy, Group, GroupCommandsExt, GroupMember},
    assets::{AssetState, MainAssets},
    map::{HexCoord, MapEvent, PresenceLayer},
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_sprite3d::{Sprite3d, Sprite3dBundle, Sprite3dParams};
use core::{ops::Range, time::Duration};
use rand::Rng;
use smallvec::SmallVec;

mod floating_text;
use floating_text::{float_and_fade, spawn_floating_text};
pub use floating_text::{FloatingTextAlignment, FloatingTextPrototype, FloatingTextSource};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CombatEvent>()
            .register_type::<Attack>()
            .register_type::<Health>()
            .add_systems(
                Update,
                initiate_combat
                    .run_if(on_event::<MapEvent>())
                    .in_set(ActionSet::PostApply),
            )
            .add_systems(
                Update,
                (
                    combat_round.run_if(on_timer(Duration::from_millis(600))),
                    combat_log,
                    spawn_damage_text,
                    despawn_no_health.after(combat_round),
                    finish_combat.after(despawn_no_health),
                    float_and_fade,
                    spawn_floating_text.run_if(on_timer(Duration::from_millis(100))),
                )
                    .run_if(in_state(AssetState::Loaded)),
            );
    }
}

#[derive(Component)]
pub struct Combat {
    position: HexCoord,
    initiative_order: SmallVec<[Entity; 8]>,
    initiative: usize,
}

#[derive(Bundle)]
pub struct CombatBundle {
    combat: Combat,
    sprite3d: Sprite3dBundle,
    floating_text_source: FloatingTextSource,
}

pub type CombatParams<'w, 's> = (Res<'w, MainAssets>, Sprite3dParams<'w, 's>);

impl CombatBundle {
    pub fn new(
        (main_assets, sprite_params): &mut CombatParams,
        position: HexCoord,
        initiative_order: SmallVec<[Entity; 8]>,
    ) -> Self {
        Self {
            combat: Combat {
                position,
                initiative: 0,
                initiative_order,
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
            floating_text_source: FloatingTextSource::default(),
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Health(pub u16, pub u16);

impl Health {
    pub fn heal(&mut self, amount: u16) -> u16 {
        let healed = (self.1 - self.0).min(amount);
        self.0 += healed;
        healed
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Attack {
    pub low: u16,
    pub high: u16,
}

impl Attack {
    pub fn range(&self) -> Range<u16> {
        self.low..self.high
    }
}

#[derive(Event)]
pub enum CombatEvent {
    Initiate(Entity),
    FriendDamage(Entity, u16),
    EnemyDamage(Entity, u16),
}

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

pub fn despawn_no_health(
    mut commands: Commands,
    health_query: Query<(Entity, &Health, Option<&GroupMember>)>,
) {
    for (entity, health, maybe_member) in &health_query {
        if health.0 == 0 {
            info!("{:?} is dead", entity);
            if let Some(group) = maybe_member.and_then(|member| member.group) {
                commands.entity(group).remove_members(&[entity]);
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
