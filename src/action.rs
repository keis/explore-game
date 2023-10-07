use crate::{
    actor::{
        Group, GroupCommandsExt, Movement, Party, PartyBundle, PartyParams, Slide, SlideEvent,
    },
    combat::{Combat, CombatEvent},
    map::{
        HexCoord, MapCommandsExt, MapPresence, Offset, PathFinder, PathGuided, PresenceLayer,
        ZoneLayer,
    },
    scene::save,
    structure::{Camp, CampBundle, CampParams, Portal},
    terrain::{CrystalDeposit, Terrain},
    turn::{set_player_turn, TurnState},
};
use bevy::prelude::*;
use smallvec::SmallVec;
use std::collections::VecDeque;

#[derive(Clone, Debug, Event)]
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
    CollectCrystals(Entity),
    OpenPortal(Entity),
}

pub struct ActionPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ActionSet {
    Prepare,
    Apply,
    CommandFlush,
    PostApply,
    FollowUp,
    Cleanup,
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameAction>()
            .insert_resource(GameActionQueue::default())
            .configure_sets(
                Update,
                (
                    ActionSet::Prepare,
                    ActionSet::Apply,
                    ActionSet::CommandFlush,
                    ActionSet::PostApply,
                    ActionSet::FollowUp,
                    ActionSet::Cleanup,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                advance_action_queue
                    .run_if(ready_for_next_action)
                    .in_set(ActionSet::Prepare),
            )
            .add_systems(
                Update,
                (
                    handle_move.run_if(has_current_action),
                    handle_enemy_move.run_if(has_current_action),
                    handle_move_to.run_if(has_current_action),
                    handle_resume_move.run_if(has_current_action),
                    handle_make_camp.run_if(has_current_action),
                    handle_break_camp.run_if(has_current_action),
                    handle_enter_camp.run_if(has_current_action),
                    handle_create_party_from_camp.run_if(has_current_action),
                    handle_split_party.run_if(has_current_action),
                    handle_merge_party.run_if(has_current_action),
                    handle_collect_crystals.run_if(has_current_action),
                    handle_open_portal.run_if(has_current_action),
                    handle_slide_stopped
                        .run_if(on_event::<SlideEvent>())
                        .after(handle_move),
                )
                    .in_set(ActionSet::Apply),
            )
            .add_systems(
                Update,
                (
                    apply_deferred.in_set(ActionSet::CommandFlush),
                    follow_path
                        .run_if(has_current_action)
                        .in_set(ActionSet::FollowUp),
                    clear_current_action
                        .run_if(has_current_action)
                        .in_set(ActionSet::Cleanup),
                    set_player_turn
                        .run_if(in_state(TurnState::System))
                        .run_if(action_queue_is_empty),
                ),
            );
    }
}

#[derive(Default, Resource)]
pub struct GameActionQueue {
    deque: VecDeque<GameAction>,
    current: Option<GameAction>,
    waiting: bool,
}

impl GameActionQueue {
    pub fn add(&mut self, action: GameAction) {
        self.deque.push_back(action);
    }

    pub fn is_waiting(&self) -> bool {
        self.waiting
    }

    pub fn has_next(&self) -> bool {
        !self.deque.is_empty()
    }

    pub fn start_next(&mut self) {
        self.current = self.deque.pop_front();
    }

    pub fn wait(&mut self) {
        self.waiting = true;
    }

    pub fn done(&mut self) {
        self.waiting = false;
    }

    pub fn clear(&mut self) {
        self.current = None;
    }
}

pub fn advance_action_queue(mut game_action_queue: ResMut<GameActionQueue>) {
    game_action_queue.start_next();
}

pub fn clear_current_action(mut game_action_queue: ResMut<GameActionQueue>) {
    game_action_queue.clear();
}

pub fn has_current_action(game_action_queue: Res<GameActionQueue>) -> bool {
    !game_action_queue.is_waiting() && game_action_queue.current.is_some()
}

pub fn action_queue_is_empty(game_action_queue: Res<GameActionQueue>) -> bool {
    game_action_queue.current.is_none() && !game_action_queue.has_next()
}

pub fn ready_for_next_action(
    game_action_queue: Res<GameActionQueue>,
    combat_query: Query<&Combat>,
) -> bool {
    !game_action_queue.is_waiting()
        && (game_action_queue.current.is_some() || game_action_queue.has_next())
        && combat_query.is_empty()
}

pub fn handle_move(
    mut queue: ResMut<GameActionQueue>,
    mut party_query: Query<(&Group, &mut Movement, &mut Slide, &Transform, &Offset), With<Party>>,
    mut member_movement_query: Query<&mut Movement, Without<Party>>,
) {
    let Some(GameAction::Move(e, next)) = queue.current else { return };
    let Ok((group, mut movement, mut slide, transform, offset)) = party_query.get_mut(e) else {
        return;
    };

    if movement.points == 0 {
        warn!("tried to move without movement points");
        queue.clear();
        return;
    }

    movement.points -= 1;
    let mut iter = member_movement_query.iter_many_mut(&group.members);
    while let Some(mut movement) = iter.fetch_next() {
        movement.points -= 1;
    }

    slide.start = transform.translation;
    slide.end = Vec3::from(next) + offset.0;
    slide.progress = 0.0;

    queue.wait();
}

fn handle_enemy_move(
    mut queue: ResMut<GameActionQueue>,
    mut enemy_query: Query<(&mut Slide, &Transform, &Offset), Without<Party>>,
) {
    let Some(GameAction::Move(e, next)) = queue.current else { return };
    let Ok((mut slide, transform, offset)) = enemy_query.get_mut(e) else { return };

    slide.start = transform.translation;
    slide.end = Vec3::from(next) + offset.0;
    slide.progress = 0.0;

    queue.wait();
}

pub fn handle_slide_stopped(
    mut commands: Commands,
    mut events: EventReader<SlideEvent>,
    mut queue: ResMut<GameActionQueue>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(map_entity) = map_query.get_single() else { return };
    for _ in &mut events {
        let Some(GameAction::Move(e, next)) = queue.current else { return };
        queue.done();

        commands.entity(map_entity).move_presence(e, next);
    }
}

pub fn follow_path(
    mut queue: ResMut<GameActionQueue>,
    mut combat_events: EventReader<CombatEvent>,
    mut path_guided_query: Query<(&Movement, &mut PathGuided)>,
) {
    let Some(GameAction::Move(e, ..)) = queue.current else { return };
    let Ok((party_movement, mut pathguided)) = path_guided_query.get_mut(e) else { return };

    pathguided.advance();

    if combat_events.iter().count() > 0 {
        return;
    }

    // Keep moving if a path is set
    if party_movement.points > 0 {
        if let Some(next) = pathguided.next() {
            queue.add(GameAction::Move(e, *next));
        }
    }
}

pub fn handle_move_to(
    mut queue: ResMut<GameActionQueue>,
    mut presence_query: Query<(&MapPresence, &Movement, &mut PathGuided)>,
    path_finder: PathFinder,
) {
    let Some(GameAction::MoveTo(e, goal)) = queue.current else { return };

    let Ok((presence, party_movement, mut pathguided)) = presence_query.get_mut(e) else { return };
    let Some((path, _length)) = path_finder.find_path(presence.position, goal) else { return };
    pathguided.path(path);
    if party_movement.points > 0 {
        if let Some(next) = pathguided.next() {
            queue.add(GameAction::Move(e, *next));
        }
    }
}

pub fn handle_resume_move(
    mut queue: ResMut<GameActionQueue>,
    path_guided_query: Query<&PathGuided>,
) {
    let Some(GameAction::ResumeMove(e)) = queue.current else { return };

    let Ok(pathguided) = path_guided_query.get(e) else { return };

    if let Some(next) = pathguided.next() {
        queue.add(GameAction::Move(e, *next));
    }
}

pub fn handle_make_camp(
    queue: ResMut<GameActionQueue>,
    mut commands: Commands,
    mut spawn_camp_params: CampParams,
    map_query: Query<(Entity, &ZoneLayer, &PresenceLayer)>,
    terrain_query: Query<&Terrain>,
    mut party_query: Query<(&mut Party, &Group, &MapPresence)>,
    camp_query: Query<&Camp>,
) {
    let Some(GameAction::MakeCamp(party_entity)) = queue.current else { return };

    let Ok((mut party, group, presence)) = party_query.get_mut(party_entity) else { return };
    let Ok((map_entity, zone_layer, presence_layer)) = map_query.get_single() else { return };
    let Some(terrain) = zone_layer.get(presence.position).and_then(|&e| terrain_query.get(e).ok()) else { return };

    if *terrain == Terrain::Mountain {
        info!("Can't camp here");
        return;
    }

    let position = presence.position;
    if camp_query
        .iter_many(presence_layer.presence(position))
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

    info!("Spawning camp at {}", position);
    party.supplies -= 1;
    commands
        .entity(map_entity)
        .with_presence(position, |location| {
            location
                .spawn((
                    Name::new("Camp"),
                    save::Save,
                    CampBundle::new(
                        position,
                        Camp {
                            name: String::from("New camp"),
                            supplies: party.supplies,
                            crystals: party.crystals,
                        },
                    )
                    .with_fluff(&mut spawn_camp_params),
                ))
                .add_members(&group.members);
        });
}

pub fn handle_break_camp(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_query: Query<(&mut Party, &MapPresence)>,
    map_query: Query<(Entity, &PresenceLayer)>,
    camp_query: Query<(Entity, &Camp, &Group)>,
) {
    let Some(GameAction::BreakCamp(e)) = queue.current else { return };

    let Ok((mut party, presence)) = party_query.get_mut(e) else { return };
    let Ok((map_entity, presence_layer)) = map_query.get_single() else { return };
    let Some((camp_entity, camp, group)) = camp_query.iter_many(presence_layer.presence(presence.position)).next() else { return };
    if !group.members.is_empty() {
        info!("Camp is not empty");
        return;
    }
    info!("Depawning camp at {}", presence.position);
    party.supplies += camp.supplies + 1;
    commands.entity(map_entity).despawn_presence(camp_entity);
}

pub fn handle_enter_camp(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_query: Query<(&mut Party, &Group)>,
    mut camp_query: Query<&mut Camp>,
) {
    let Some(GameAction::EnterCamp(party_entity, camp_entity)) = queue.current else { return };

    let Ok((mut party, group)) = party_query.get_mut(party_entity) else { return };
    let Ok(mut camp) = camp_query.get_mut(camp_entity) else { return };
    camp.supplies += party.supplies;
    party.supplies = 0;
    camp.crystals += party.crystals;
    party.crystals = 0;
    commands.entity(camp_entity).add_members(&group.members);
}

pub fn handle_create_party_from_camp(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_params: PartyParams,
    mut camp_query: Query<(&mut Camp, &MapPresence)>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Some(GameAction::CreatePartyFromCamp(camp_entity, characters)) = &queue.current else { return };

    info!("Creating party at camp {:?} {:?}", camp_entity, characters);
    let (mut camp, presence) = camp_query.get_mut(*camp_entity).unwrap();
    let Ok(map_entity) = map_query.get_single() else { return };
    let new_supplies = if camp.supplies > 0 {
        camp.supplies -= 1;
        1
    } else {
        0
    };
    commands
        .entity(map_entity)
        .with_presence(presence.position, |location| {
            location
                .spawn((
                    Name::new("Party"),
                    save::Save,
                    PartyBundle::new(presence.position, "New Party".to_string(), new_supplies)
                        .with_fluff(&mut party_params),
                ))
                .add_members(characters);
        });
}

pub fn handle_split_party(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_params: PartyParams,
    mut party_query: Query<(&mut Party, &Group, &MapPresence)>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Some(GameAction::SplitParty(party_entity, characters)) = &queue.current else { return };

    let (mut party, group, presence) = party_query.get_mut(*party_entity).unwrap();
    let Ok(map_entity) = map_query.get_single() else { return };
    if group.members.len() == characters.len() {
        info!("Refusing split resulting in empty party");
        return;
    }
    let new_supplies = if party.supplies > 1 {
        party.supplies -= 1;
        1
    } else {
        0
    };
    commands
        .entity(map_entity)
        .with_presence(presence.position, |location| {
            location
                .spawn((
                    Name::new("Party"),
                    save::Save,
                    PartyBundle::new(presence.position, "New Party".to_string(), new_supplies)
                        .with_fluff(&mut party_params),
                ))
                .add_members(characters);
        });
}

pub fn handle_merge_party(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_query: Query<(&mut Party, &Group, &MapPresence)>,
) {
    let Some(GameAction::MergeParty(parties)) = &queue.current else { return };

    let [target, rest @ ..] = parties.as_slice() else { return };
    let target_position = party_query
        .get(*target)
        .map(|(_, _, p)| p.position)
        .unwrap();
    let mut characters = SmallVec::<[Entity; 8]>::new();
    let mut supplies = 0;
    let mut crystals = 0;
    let mut iter = party_query.iter_many_mut(rest);
    while let Some((mut party, group, presence)) = iter.fetch_next() {
        if presence.position != target_position {
            info!("Skipping party in other location");
            continue;
        }
        supplies += party.supplies;
        party.supplies = 0;
        crystals += party.crystals;
        party.crystals = 0;
        characters.append(&mut group.members.clone());
    }
    let (mut party, _, _) = party_query.get_mut(*target).unwrap();
    party.supplies += supplies;
    party.crystals += crystals;
    commands.entity(*target).add_members(&characters);
}

pub fn handle_collect_crystals(
    queue: ResMut<GameActionQueue>,
    mut party_query: Query<(&mut Party, &MapPresence)>,
    map_query: Query<&ZoneLayer>,
    mut crystal_deposit_query: Query<&mut CrystalDeposit>,
) {
    let Some(GameAction::CollectCrystals(party_entity)) = queue.current else { return };

    let Ok((mut party, presence)) = party_query.get_mut(party_entity) else { return };
    let Ok(zone_layer) = map_query.get_single() else { return };
    let Some(mut crystal_deposit) = zone_layer
        .get(presence.position)
        .and_then(|&e| crystal_deposit_query.get_mut(e).ok()) else {
            info!("No crystal deposit here");
            return;
        };

    party.crystals += crystal_deposit.take() as u32
}

pub fn handle_open_portal(
    queue: ResMut<GameActionQueue>,
    party_query: Query<&MapPresence, With<Party>>,
    map_query: Query<&PresenceLayer>,
    mut portal_query: Query<&mut Portal>,
) {
    let Some(GameAction::OpenPortal(party_entity)) = queue.current else { return };

    let Ok(presence) = party_query.get(party_entity) else { return };
    let Ok(presence_layer) = map_query.get_single() else { return };
    let mut portal_iter = portal_query.iter_many_mut(presence_layer.presence(presence.position));
    let Some(mut portal) = portal_iter.fetch_next() else { return };

    if !portal.open {
        portal.open = true;
    }
}
