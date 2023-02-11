use crate::{
    assets::MainAssets,
    camp::Camp,
    character::Movement,
    hex::coord_to_vec3,
    map::{
        find_path, AddMapPresence, DespawnPresence, GameMap, HexCoord, MapPresence,
        MoveMapPresence, Offset, PathGuided, ViewRadius, Zone,
    },
    party::{spawn_party, Group, JoinGroup, Party},
    slide::{Slide, SlideEvent},
    turn::Turn,
    State, VIEW_RADIUS,
};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use smallvec::SmallVec;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum GameAction {
    Move(Entity, HexCoord),
    MoveTo(Entity, HexCoord),
    ResumeMove(Entity),
    MakeCamp(Entity),
    BreakCamp(Entity),
    SplitParty(Entity, SmallVec<[Entity; 8]>),
    Save(),
}

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameAction>()
            .insert_resource(GameActionQueue::default())
            .add_system_set(
                SystemSet::on_update(State::Running)
                    .with_system(trigger_action)
                    .with_system(handle_move)
                    .with_system(handle_slide_stopped)
                    .with_system(handle_move_to)
                    .with_system(handle_resume_move)
                    .with_system(handle_make_camp)
                    .with_system(handle_break_camp)
                    .with_system(handle_split_party),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_on_save)
                    .with_system(handle_save),
            );
    }
}

#[derive(Default, Resource)]
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
            events.send(action.clone());
            queue.current = Some(action);
        }
    }

    if queue.is_changed() && !queue.is_waiting() {
        if let Some(action) = queue.deque.pop_front() {
            events.send(action.clone());
            queue.current = Some(action);
        }
    }
}

pub fn handle_move(
    mut events: EventReader<GameAction>,
    mut party_query: Query<(&Group, &mut Movement, &mut Slide, &Transform, &Offset), With<Party>>,
    mut member_movement_query: Query<&mut Movement, Without<Party>>,
    mut queue: ResMut<GameActionQueue>,
) {
    for event in events.iter() {
        if let GameAction::Move(e, next) = event {
            if let Ok((group, mut movement, mut slide, transform, offset)) = party_query.get_mut(*e)
            {
                if movement.points == 0 {
                    warn!("tried to move without movement points");
                    queue.current.take();
                    continue;
                }
                movement.points -= 1;
                let mut iter = member_movement_query.iter_many_mut(&group.members);
                while let Some(mut movement) = iter.fetch_next() {
                    movement.points -= 1;
                }
                slide.start = transform.translation;
                slide.end = coord_to_vec3(*next) + offset.0;
                slide.progress = 0.0;
            }
        }
    }
}

pub fn handle_slide_stopped(
    mut commands: Commands,
    mut events: EventReader<SlideEvent>,
    mut queue: ResMut<GameActionQueue>,
    mut presence_query: Query<(&MapPresence, &Movement, &mut PathGuided)>,
) {
    for _ in events.iter() {
        if let Some(last_action) = queue.current.take() {
            if let GameAction::Move(e, next) = last_action {
                let (presence, party_movement, mut pathguided) = presence_query.get_mut(e).unwrap();
                info!("done with move action {:?}", last_action);
                commands.add(MoveMapPresence {
                    map: presence.map,
                    presence: e,
                    position: next,
                });
                pathguided.advance();
                // Keep moving if a path is set
                if party_movement.points > 0 {
                    if let Some(next) = pathguided.next() {
                        queue.add(GameAction::Move(e, *next));
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
    mut presence_query: Query<(&MapPresence, &Movement, &mut PathGuided)>,
    zone_query: Query<&Zone>,
    map_query: Query<&GameMap>,
) {
    // Use let_chains after rust 1.64
    for event in events.iter() {
        if let GameAction::MoveTo(e, goal) = event {
            queue.current.take();
            if let Ok((presence, party_movement, mut pathguided)) = presence_query.get_mut(*e) {
                if let Ok(map) = map_query.get(presence.map) {
                    if let Some((path, _length)) =
                        find_path(map, &zone_query, presence.position, *goal)
                    {
                        pathguided.path(path);
                        if party_movement.points > 0 {
                            if let Some(next) = pathguided.next() {
                                queue.add(GameAction::Move(*e, *next));
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_resume_move(
    mut events: EventReader<GameAction>,
    mut queue: ResMut<GameActionQueue>,
    path_guided_query: Query<&PathGuided>,
) {
    for event in events.iter() {
        if let GameAction::ResumeMove(e) = event {
            if let Ok(pathguided) = path_guided_query.get(*e) {
                if let Some(next) = pathguided.next() {
                    info!("Resuming move!");
                    queue.add(GameAction::Move(*e, *next));
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
    mut map_query: Query<(Entity, &GameMap)>,
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
                    .iter_many(map.presence(position))
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
                    .spawn((
                        PbrBundle {
                            mesh: assets.tent_mesh.clone(),
                            material: standard_materials
                                .add(Color::rgb(0.631, 0.596, 0.165).into()),
                            transform: Transform::from_translation(coord_to_vec3(position))
                                .with_rotation(Quat::from_rotation_y(1.0)),
                            ..default()
                        },
                        PickableBundle::default(),
                        Camp {
                            name: String::from("New Camp"),
                        },
                        Offset(Vec3::ZERO),
                        ViewRadius(VIEW_RADIUS),
                    ))
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
    mut map_query: Query<(Entity, &GameMap)>,
    camp_query: Query<Entity, With<Camp>>,
) {
    for event in events.iter() {
        if let GameAction::BreakCamp(e) = event {
            if let Ok((mut party, presence)) = party_query.get_mut(*e) {
                let (map_entity, map) = map_query
                    .get_mut(presence.map)
                    .expect("references valid map");

                let position = presence.position;
                let maybe_camp = camp_query.iter_many(map.presence(position)).next();
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

pub fn handle_split_party(
    mut commands: Commands,
    mut spawn_party_params: ParamSet<(Res<MainAssets>, ResMut<Assets<StandardMaterial>>)>,
    mut events: EventReader<GameAction>,
    mut party_query: Query<(&mut Party, &Group, &MapPresence)>,
) {
    for event in events.iter() {
        let GameAction::SplitParty(party_entity, characters) = event else { continue };

        let (mut party, group, presence) = party_query.get_mut(*party_entity).unwrap();
        if group.members.len() == characters.len() {
            info!("Refusing split resulting in empty party");
            continue;
        }
        let new_supplies = if party.supplies > 1 {
            party.supplies -= 1;
            1
        } else {
            0
        };
        let new_party = spawn_party(
            &mut commands,
            &mut spawn_party_params,
            presence.position,
            "New Party".to_string(),
            new_supplies,
        );
        commands.add(AddMapPresence {
            map: presence.map,
            presence: new_party,
            position: presence.position,
        });
        commands.add(JoinGroup {
            group: new_party,
            members: characters.clone(),
        });
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
