use crate::{
    action::ActionSet,
    actor::{
        character::Character,
        enemy::Enemy,
        party::{Group, GroupCommandsExt, GroupMember},
    },
    assets::{AssetState, MainAssets},
    interface::InterfaceAssets,
    map::{HexCoord, MapEvent, PresenceLayer},
};
use bevy::prelude::*;
use bevy_mod_billboard::prelude::*;
use bevy_sprite3d::{Sprite3d, Sprite3dBundle, Sprite3dParams};
use core::{ops::Range, time::Duration};
use rand::Rng;
use smallvec::SmallVec;

mod floating_text;
use floating_text::{float_and_fade, FloatingText};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CombatEvent>()
            .add_systems(
                Update,
                initiate_combat
                    .run_if(on_event::<MapEvent>())
                    .in_set(ActionSet::PostApply),
            )
            .add_systems(Update, float_and_fade)
            .add_systems(
                Update,
                (
                    combat_round,
                    combat_log,
                    spawn_damage_text,
                    despawn_no_health.after(combat_round),
                    finish_combat.after(despawn_no_health),
                )
                    .run_if(in_state(AssetState::Loaded)),
            );
    }
}

#[derive(Component)]
pub struct Combat {
    position: HexCoord,
    timer: Timer,
    initiative_order: SmallVec<[Entity; 8]>,
    initiative: usize,
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
        initiative_order: SmallVec<[Entity; 8]>,
    ) -> Self {
        Self {
            combat: Combat {
                position,
                timer: Timer::new(Duration::from_millis(600), TimerMode::Repeating),
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
        }
    }
}

#[derive(Component, Default)]
pub struct Health(pub u16, pub u16);

impl Health {
    pub fn heal(&mut self, amount: u16) {
        self.0 = (self.0 + amount).min(self.1);
    }
}

#[derive(Component, Default)]
pub struct Attack(pub Range<u16>);

#[derive(Event)]
pub enum CombatEvent {
    Initiate(Entity),
    FriendDamage(Entity, u16),
    EnemyDamage(Entity, u16),
}

pub fn combat_log(mut combat_events: EventReader<CombatEvent>, combat_query: Query<&Combat>) {
    for event in &mut combat_events {
        match event {
            CombatEvent::Initiate(entity) => {
                let Ok(combat) = combat_query.get(*entity) else { continue };
                info!("Combat initiated at {}!", combat.position)
            }
            CombatEvent::FriendDamage(entity, damage) => {
                let Ok(combat) = combat_query.get(*entity) else { continue };
                info!(
                    "Damage to friendly in combat at {} - {}",
                    combat.position, damage
                )
            }
            CombatEvent::EnemyDamage(entity, damage) => {
                let Ok(combat) = combat_query.get(*entity) else { continue };
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
    let Ok(presence_layer) = map_query.get_single() else { return };
    for event in &mut map_events {
        let MapEvent::PresenceMoved { position, .. } = event else { continue };
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
    time: Res<Time>,
    attacker_query: Query<(&Attack, Option<&Enemy>)>,
    mut target_query: Query<(&mut Health, Option<&Enemy>)>,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut combat) in &mut combat_query {
        combat.timer.tick(time.delta());
        if !combat.timer.just_finished() {
            continue;
        }

        info!("Combat at {}", combat.position);

        let attacker_entity = combat.initiative_order[combat.initiative];
        combat.initiative = (combat.initiative + 1) % combat.initiative_order.len();

        let Ok((attack, maybe_attacker_enemy)) = attacker_query.get(attacker_entity) else { continue };
        let mut target_iter = target_query.iter_many_mut(&combat.initiative_order);
        while let Some((mut health, maybe_target_enemy)) = target_iter.fetch_next() {
            if health.0 == 0 || maybe_attacker_enemy.is_some() == maybe_target_enemy.is_some() {
                continue;
            }
            let damage = rng.gen_range(attack.0.clone()).min(health.0);
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
    mut commands: Commands,
    mut combat_events: EventReader<CombatEvent>,
    combat_query: Query<&Combat>,
    interface_assets: Res<InterfaceAssets>,
) {
    for event in &mut combat_events {
        match event {
            CombatEvent::FriendDamage(entity, damage) => {
                let Ok(combat) = combat_query.get(*entity) else { continue };
                commands.spawn((
                    FloatingText::default(),
                    BillboardTextBundle {
                        transform: Transform::from_translation(
                            Vec3::from(combat.position) + Vec3::new(-0.1, 1.0, 0.2),
                        )
                        .with_scale(Vec3::new(0.01, 0.01, 0.01)),
                        text: Text::from_sections([TextSection {
                            value: damage.to_string(),
                            style: TextStyle {
                                font_size: 26.0,
                                font: interface_assets.font.clone(),
                                color: Color::RED,
                            },
                        }])
                        .with_alignment(TextAlignment::Center),
                        ..default()
                    },
                ));
            }
            CombatEvent::EnemyDamage(entity, damage) => {
                let Ok(combat) = combat_query.get(*entity) else { continue };
                commands.spawn((
                    FloatingText::default(),
                    BillboardTextBundle {
                        transform: Transform::from_translation(
                            Vec3::from(combat.position) + Vec3::new(0.1, 1.0, 0.2),
                        )
                        .with_scale(Vec3::new(0.01, 0.01, 0.01)),
                        text: Text::from_sections([TextSection {
                            value: damage.to_string(),
                            style: TextStyle {
                                font_size: 26.0,
                                font: interface_assets.font.clone(),
                                color: Color::YELLOW,
                            },
                        }])
                        .with_alignment(TextAlignment::Center),
                        ..default()
                    },
                ));
            }
            _ => {}
        }
    }
}
