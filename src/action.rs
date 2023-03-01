use crate::{
    assets::MainAssets,
    camp::{spawn_camp, Camp},
    character::Movement,
    map::{
        coord_to_vec3, AddMapPresence, DespawnPresence, GameMap, HexCoord, MapPresence,
        MoveMapPresence, Offset, PathFinder, PathGuided,
    },
    party::{spawn_party, Group, JoinGroup, Party},
    slide::{Slide, SlideEvent},
    State,
};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use smallvec::SmallVec;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum GameAction {
    Move(Entity, HexCoord),
    MoveTo(Entity, HexCoord),
    ResumeMove(Entity),
    MakeCamp(Entity),
    BreakCamp(Entity),
    EnterCamp(Entity, Entity),
    SplitParty(Entity, SmallVec<[Entity; 8]>),
    MergeParty(SmallVec<[Entity; 8]>),
    CreatePartyFromCamp(Entity, SmallVec<[Entity; 8]>),
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
                    .with_system(handle_enemy_move)
                    .with_system(handle_slide_stopped)
                    .with_system(handle_move_to)
                    .with_system(handle_resume_move)
                    .with_system(handle_make_camp)
                    .with_system(handle_break_camp)
                    .with_system(handle_enter_camp)
                    .with_system(handle_create_party_from_camp)
                    .with_system(handle_split_party)
                    .with_system(handle_merge_party),
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

pub fn trigger_action(mut events: EventWriter<GameAction>, mut queue: ResMut<GameActionQueue>) {
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
        let GameAction::Move(e, next) = event else { continue };
        let Ok((group, mut movement, mut slide, transform, offset)) = party_query.get_mut(*e) else { continue };

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

fn handle_enemy_move(
    mut events: EventReader<GameAction>,
    mut enemy_query: Query<(&mut Slide, &Transform, &Offset), Without<Party>>,
) {
    for event in events.iter() {
        let GameAction::Move(e, next) = event else { continue };
        let Ok((mut slide, transform, offset)) = enemy_query.get_mut(*e) else { continue };

        slide.start = transform.translation;
        slide.end = coord_to_vec3(*next) + offset.0;
        slide.progress = 0.0;
    }
}

pub fn handle_slide_stopped(
    mut commands: Commands,
    mut events: EventReader<SlideEvent>,
    mut queue: ResMut<GameActionQueue>,
    mut presence_query: Query<(&MapPresence, Option<(&Movement, &mut PathGuided)>)>,
) {
    for _ in events.iter() {
        let Some(last_action) = queue.current.take() else {
            warn!("Slide finished for some unknown action");
            continue
        };
        let GameAction::Move(e, next) = last_action else { continue };
        let Ok((presence, optional)) = presence_query.get_mut(e) else { continue };

        commands.add(MoveMapPresence {
            map: presence.map,
            presence: e,
            position: next,
        });

        if let Some((party_movement, mut pathguided)) = optional {
            pathguided.advance();
            // Keep moving if a path is set
            if party_movement.points > 0 {
                if let Some(next) = pathguided.next() {
                    queue.add(GameAction::Move(e, *next));
                }
            }
        }
    }
}

pub fn handle_move_to(
    mut events: EventReader<GameAction>,
    mut queue: ResMut<GameActionQueue>,
    mut presence_query: Query<(&MapPresence, &Movement, &mut PathGuided)>,
    path_finder: PathFinder,
) {
    for event in events.iter() {
        let GameAction::MoveTo(e, goal) = event else { continue };
        let Ok((presence, party_movement, mut pathguided)) = presence_query.get_mut(*e) else { continue };

        queue.current.take();
        let Some((path, _length)) = path_finder.find_path(presence.position, *goal) else { continue };
        pathguided.path(path);
        if party_movement.points > 0 {
            if let Some(next) = pathguided.next() {
                queue.add(GameAction::Move(*e, *next));
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
        let GameAction::ResumeMove(e) = event else { continue };
        let Ok(pathguided) = path_guided_query.get(*e) else { continue };

        if let Some(next) = pathguided.next() {
            info!("Resuming move!");
            queue.add(GameAction::Move(*e, *next));
        }
    }
}

pub fn handle_make_camp(
    mut commands: Commands,
    mut spawn_camp_params: ParamSet<(Res<MainAssets>, ResMut<Assets<StandardMaterial>>)>,
    mut events: EventReader<GameAction>,
    map_query: Query<(Entity, &GameMap)>,
    mut party_query: Query<(&mut Party, &Group, &MapPresence)>,
    camp_query: Query<&Camp>,
) {
    for event in events.iter() {
        let GameAction::MakeCamp(party_entity) = event else { continue };
        let Ok((mut party, group, presence)) = party_query.get_mut(*party_entity) else { continue };
        let Ok((map_entity, map)) = map_query.get(presence.map) else { continue };

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
        let entity = spawn_camp(
            &mut commands,
            &mut spawn_camp_params,
            position,
            String::from("New camp"),
        );
        commands.add(AddMapPresence {
            map: map_entity,
            presence: entity,
            position,
        });
        commands.add(JoinGroup {
            group: entity,
            members: group.members.clone(),
        });
    }
}

pub fn handle_break_camp(
    mut commands: Commands,
    mut events: EventReader<GameAction>,
    mut party_query: Query<(&mut Party, &MapPresence)>,
    mut map_query: Query<(Entity, &GameMap)>,
    camp_query: Query<(Entity, &Camp, &Group)>,
) {
    for event in events.iter() {
        if let GameAction::BreakCamp(e) = event {
            if let Ok((mut party, presence)) = party_query.get_mut(*e) {
                let (map_entity, map) = map_query
                    .get_mut(presence.map)
                    .expect("references valid map");

                let position = presence.position;
                if let Some((camp_entity, camp, group)) =
                    camp_query.iter_many(map.presence(position)).next()
                {
                    if !group.members.is_empty() {
                        info!("Camp is not empty");
                        continue;
                    }
                    info!("Depawning camp at {:?}", position);
                    party.supplies += camp.supplies + 1;
                    commands.add(DespawnPresence {
                        map: map_entity,
                        presence: camp_entity,
                    });
                }
            }
        }
    }
}

pub fn handle_enter_camp(
    mut commands: Commands,
    mut events: EventReader<GameAction>,
    mut party_query: Query<(&mut Party, &Group)>,
    mut camp_query: Query<&mut Camp>,
) {
    for event in events.iter() {
        let GameAction::EnterCamp(party_entity, camp_entity) = event else { continue };
        let Ok((mut party, group)) = party_query.get_mut(*party_entity) else { continue };
        let Ok(mut camp) = camp_query.get_mut(*camp_entity) else { continue };
        camp.supplies += party.supplies;
        party.supplies = 0;
        commands.add(JoinGroup {
            group: *camp_entity,
            members: group.members.clone(),
        });
    }
}

pub fn handle_create_party_from_camp(
    mut commands: Commands,
    mut spawn_party_params: ParamSet<(Res<MainAssets>, ResMut<Assets<StandardMaterial>>)>,
    mut events: EventReader<GameAction>,
    mut camp_query: Query<(&mut Camp, &MapPresence)>,
) {
    for event in events.iter() {
        let GameAction::CreatePartyFromCamp(camp_entity, characters) = event else { continue };
        info!("Creating party at camp {:?} {:?}", camp_entity, characters);
        let (mut camp, presence) = camp_query.get_mut(*camp_entity).unwrap();
        let new_supplies = if camp.supplies > 0 {
            camp.supplies -= 1;
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

pub fn handle_merge_party(
    mut commands: Commands,
    mut events: EventReader<GameAction>,
    mut party_query: Query<(&mut Party, &Group, &MapPresence)>,
) {
    for event in events.iter() {
        let GameAction::MergeParty(parties) = event else { continue };
        let [target, rest @ ..] = parties.as_slice() else { continue };
        let target_position = party_query
            .get(*target)
            .map(|(_, _, p)| p.position)
            .unwrap();
        let mut characters = SmallVec::<[Entity; 8]>::new();
        let mut supplies = 0;
        let mut iter = party_query.iter_many_mut(rest);
        while let Some((mut party, group, presence)) = iter.fetch_next() {
            if presence.position != target_position {
                info!("Skipping party in other location");
                continue;
            }
            supplies += party.supplies;
            party.supplies = 0;
            characters.append(&mut group.members.clone());
        }
        let (mut party, _, _) = party_query.get_mut(*target).unwrap();
        party.supplies += supplies;
        commands.add(JoinGroup {
            group: *target,
            members: characters,
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
