use crate::assets::MainAssets;
use crate::camp::Camp;
use crate::hex::coord_to_vec3;
use crate::map::{
    find_path, AddMapPresence, DespawnPresence, HexCoord, MapComponent, MapPresence,
    MoveMapPresence, Offset, PathGuided, ViewRadius,
};
use crate::party::Party;
use crate::slide::{Slide, SlideEvent};
use crate::turn::Turn;
use crate::zone::{Terrain, Zone};
use crate::VIEW_RADIUS;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug)]
pub enum GameAction {
    Move(Entity, HexCoord),
    MoveTo(Entity, HexCoord),
    MakeCamp(Entity),
    BreakCamp(Entity),
    Save(),
}

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameAction>()
            .insert_resource(GameActionQueue::default())
            .add_system(trigger_action)
            .add_system(handle_move)
            .add_system(handle_slide_stopped)
            .add_system(handle_move_to)
            .add_system(handle_make_camp)
            .add_system(handle_break_camp)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_on_save)
                    .with_system(handle_save.exclusive_system()),
            );
    }
}

#[derive(Default)]
pub struct GameActionQueue {
    deque: VecDeque<GameAction>,
    current: Option<GameAction>,
}

impl GameActionQueue {
    pub fn add(&mut self, action: GameAction) {
        self.deque.push_back(action);
    }

    pub fn is_waiting(&self) -> bool {
        self.current.is_some()
    }
}

pub fn trigger_action(
    mut events: EventWriter<GameAction>,
    mut queue: ResMut<GameActionQueue>,
    turn: Res<Turn>,
) {
    if turn.is_changed() {
        if let Some(action) = queue.deque.pop_front() {
            events.send(action);
            queue.current = Some(action);
        }
    }

    if queue.is_changed() && !queue.is_waiting() {
        if let Some(action) = queue.deque.pop_front() {
            events.send(action);
            queue.current = Some(action);
        }
    }
}

pub fn handle_move(
    mut events: EventReader<GameAction>,
    map_query: Query<&MapComponent>,
    mut party_query: Query<(&mut Party, &mut Slide, &Transform, &Offset, &MapPresence)>,
) {
    for event in events.iter() {
        if let GameAction::Move(e, next) = event {
            if let Ok((mut party, mut slide, transform, offset, presence)) = party_query.get_mut(*e)
            {
                if party.movement_points == 0 {
                    warn!("tried to move without movement points");
                    continue;
                }
                let map = map_query.get(presence.map).expect("references valid map");
                party.movement_points -= 1;
                slide.start = transform.translation;
                slide.end = coord_to_vec3(*next, map.radius) + offset.0;
                slide.progress = 0.0;
            }
        }
    }
}

pub fn handle_slide_stopped(
    mut commands: Commands,
    mut events: EventReader<SlideEvent>,
    mut queue: ResMut<GameActionQueue>,
    mut presence_query: Query<(&MapPresence, &Party, &mut PathGuided)>,
) {
    for _ in events.iter() {
        if let Some(last_action) = queue.current.take() {
            if let GameAction::Move(e, next) = last_action {
                let (presence, party, mut pathguided) =
                    presence_query.get_mut(e).expect("only presence moves");
                info!("done with move action {:?}", last_action);
                commands.add(MoveMapPresence {
                    map: presence.map,
                    presence: e,
                    position: next,
                });
                // Keep moving if a path is set
                if party.movement_points > 0 {
                    if let Some(next) = pathguided.take_next() {
                        queue.add(GameAction::Move(e, next));
                    }
                }
            } else {
                warn!("Slide finished for some unknown action");
            }
        }
    }
}

pub fn handle_move_to(
    mut events: EventReader<GameAction>,
    mut queue: ResMut<GameActionQueue>,
    mut presence_query: Query<(&MapPresence, &Party, &mut PathGuided)>,
    zone_query: Query<&Zone>,
    map_query: Query<&MapComponent>,
) {
    // Use let_chains after rust 1.64
    for event in events.iter() {
        if let GameAction::MoveTo(e, goal) = event {
            queue.current.take();
            if let Ok((presence, party, mut pathguided)) = presence_query.get_mut(*e) {
                if let Ok(map) = map_query.get(presence.map) {
                    if let Some((path, _length)) =
                        find_path(presence.position, *goal, &|c: &HexCoord| {
                            if let Some(entity) = map.storage.get(*c) {
                                if let Ok(zone) = zone_query.get(entity) {
                                    return zone.terrain != Terrain::Lava;
                                }
                            }
                            false
                        })
                    {
                        pathguided.path(path);
                        if party.movement_points > 0 {
                            if let Some(next) = pathguided.take_next() {
                                queue.add(GameAction::Move(*e, next));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_make_camp(
    mut commands: Commands,
    assets: Res<MainAssets>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<GameAction>,
    mut map_query: Query<(Entity, &MapComponent)>,
    mut party_query: Query<(&mut Party, &MapPresence)>,
    camp_query: Query<&Camp>,
) {
    for event in events.iter() {
        if let GameAction::MakeCamp(e) = event {
            if let Ok((mut party, presence)) = party_query.get_mut(*e) {
                let (map_entity, map) = map_query
                    .get_mut(presence.map)
                    .expect("references valid map");

                let position = presence.position;
                if camp_query
                    .iter_many(map.storage.presence(position))
                    .next()
                    .is_some()
                {
                    info!("There's already a camp here");
                    return;
                }

                if party.supplies == 0 {
                    info!("Party does not have enough supplies to make camp");
                    return;
                }

                info!("Spawning camp at {:?}", position);
                party.supplies -= 1;
                let entity = commands
                    .spawn_bundle(PbrBundle {
                        mesh: assets.tent_mesh.clone(),
                        material: standard_materials.add(Color::rgb(0.631, 0.596, 0.165).into()),
                        transform: Transform::from_translation(coord_to_vec3(position, 1.0))
                            .with_rotation(Quat::from_rotation_y(1.0)),
                        ..default()
                    })
                    .insert_bundle(PickableBundle::default())
                    .insert(Camp {
                        name: String::from("New Camp"),
                    })
                    .insert(Offset(Vec3::ZERO))
                    .insert(ViewRadius(VIEW_RADIUS))
                    .id();
                commands.add(AddMapPresence {
                    map: map_entity,
                    presence: entity,
                    position,
                })
            }
        }
    }
}

pub fn handle_break_camp(
    mut commands: Commands,
    mut events: EventReader<GameAction>,
    mut party_query: Query<(&mut Party, &MapPresence)>,
    mut map_query: Query<(Entity, &MapComponent)>,
    camp_query: Query<Entity, With<Camp>>,
) {
    for event in events.iter() {
        if let GameAction::BreakCamp(e) = event {
            if let Ok((mut party, presence)) = party_query.get_mut(*e) {
                let (map_entity, map) = map_query
                    .get_mut(presence.map)
                    .expect("references valid map");

                let position = presence.position;
                let maybe_camp = camp_query.iter_many(map.storage.presence(position)).next();
                if let Some(camp) = maybe_camp {
                    info!("Depawning camp at {:?}", position);
                    party.supplies += 1;
                    commands.add(DespawnPresence {
                        map: map_entity,
                        presence: camp,
                    });
                }
            }
        }
    }
}

fn run_on_save(mut events: EventReader<GameAction>) -> ShouldRun {
    for event in events.iter() {
        if let GameAction::Save() = event {
            return ShouldRun::Yes;
        }
    }
    ShouldRun::No
}

pub fn handle_save(_world: &mut World) {
    info!("Save!");
}
