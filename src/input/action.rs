use super::{Deselect, NextSelectionQuery, Select, Selection};
use crate::{
    action::{GameAction, GameActionQueue},
    actor::{Character, Group, Party},
    camera::{CameraControl, CameraTarget},
    interface::InterfaceState,
    map::{MapPresence, PresenceLayer},
    path::PathGuided,
    structure::Camp,
};
use bevy::prelude::*;
pub use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Reflect, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    BreakCamp,
    Camp,
    Cancel,
    CollectCrystals,
    CreateParty,
    Deselect,
    EnterPortal,
    MergeParty,
    MultiSelect,
    NextTurn,
    OpenPortal,
    PanCamera,
    PanCameraDown,
    PanCameraLeft,
    PanCameraMotion,
    PanCameraRight,
    PanCameraUp,
    ResumeMove,
    Save,
    SelectNext,
    SplitParty,
    ToggleInspector,
    ToggleMainMenu,
    ZoomCamera,
}

pub fn magic_cancel(
    mut action_state: ResMut<ActionState<Action>>,
    interface_state: Res<State<InterfaceState>>,
    selection_query: Query<&Selection>,
) {
    let actiondata = action_state.action_data(Action::Cancel).clone();

    // Close menu
    if *interface_state == InterfaceState::Menu {
        action_state.set_action_data(Action::ToggleMainMenu, actiondata);
        return;
    }

    // Deselect
    if selection_query.iter().any(|s| s.is_selected) {
        action_state.set_action_data(Action::Deselect, actiondata);
        return;
    }

    // Open menu
    action_state.set_action_data(Action::ToggleMainMenu, actiondata);
}

pub fn handle_deselect(
    selection_query: Query<(Entity, &Selection)>,
    mut deselect_events: EventWriter<Deselect>,
) {
    for (entity, selection) in &selection_query {
        if selection.is_selected {
            deselect_events.send(Deselect(entity));
        }
    }
}

pub fn handle_enter_portal(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, _) in party_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::new_enter_portal(entity));
    }
}

pub fn handle_select_next(
    mut commands: Commands,
    next_selection_query: NextSelectionQuery,
    selection_query: Query<(Entity, &Selection, &MapPresence)>,
    camera_query: Query<Entity, With<CameraControl>>,
    mut select_events: EventWriter<Select>,
    mut deselect_events: EventWriter<Deselect>,
) {
    let camera_entity = camera_query.single();
    let Some(next) = next_selection_query.get() else {
        return;
    };
    for (entity, selection, presence) in &selection_query {
        if entity == next {
            select_events.send(Select(entity));
            commands
                .entity(camera_entity)
                .insert(CameraTarget::from_hexcoord(presence.position));
        } else if selection.is_selected {
            deselect_events.send(Deselect(entity));
        }
    }
}

pub fn handle_camp(
    party_query: Query<(Entity, &MapPresence, &Selection), With<Party>>,
    map_query: Query<&PresenceLayer>,
    camp_query: Query<Entity, With<Camp>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, presence, _) in party_query.iter().filter(|(_, _, s)| s.is_selected) {
        let Ok(presence_layer) = map_query.get_single() else {
            continue;
        };
        if let Some(camp_entity) = camp_query
            .iter_many(presence_layer.presence(presence.position))
            .next()
        {
            game_action_queue.add(GameAction::new_enter_camp(entity, camp_entity));
        } else {
            game_action_queue.add(GameAction::new_make_camp(entity));
        }
    }
}

pub fn handle_break_camp(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, _) in party_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::new_break_camp(entity));
    }
}

pub fn handle_create_party(
    camp_query: Query<(Entity, &Group, &Selection), With<Camp>>,
    character_query: Query<(Entity, &Selection), With<Character>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, group, _) in camp_query.iter().filter(|(_, _, s)| s.is_selected) {
        let selected: Vec<_> = character_query
            .iter_many(&group.members)
            .filter(|(_, s)| s.is_selected)
            .map(|(e, _)| e)
            .collect();
        if !selected.is_empty() {
            game_action_queue.add(GameAction::new_create_party_from_camp(entity, selected));
        }
    }
}

pub fn handle_split_party(
    party_query: Query<(Entity, &Group, &Selection), With<Party>>,
    character_query: Query<(Entity, &Selection), With<Character>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, group, _) in party_query.iter().filter(|(_, _, s)| s.is_selected) {
        let selected: Vec<Entity> = character_query
            .iter_many(&group.members)
            .filter(|(_, s)| s.is_selected)
            .map(|(e, _)| e)
            .collect();
        if !selected.is_empty() {
            game_action_queue.add(GameAction::new_split_party(entity, selected));
        }
    }
}

pub fn handle_merge_party(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    let mut selected_parties = party_query
        .iter()
        .filter(|(_, s)| s.is_selected)
        .map(|(e, _)| e);
    if let Some(source) = selected_parties.next() {
        game_action_queue.add(GameAction::new_merge_party(source, selected_parties));
    }
}

pub fn handle_collect_crystals(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (party, _) in party_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::new_collect_crystals(party));
    }
}

pub fn handle_open_portal(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (party, _) in party_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::new_open_portal(party));
    }
}

pub fn handle_resume_move(
    party_query: Query<(Entity, &PathGuided, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, path_guided, _) in party_query.iter().filter(|(_, _, s)| s.is_selected) {
        if let Some(next) = path_guided.next() {
            game_action_queue.add(GameAction::new_move(entity, *next));
        }
    }
}
