use super::{NextSelectionQuery, Selection};
use crate::{
    action::{GameAction, GameActionQueue},
    actor::{Character, Group, Party},
    camera::{CameraControl, CameraTarget},
    interface::MenuLayer,
    map::{MapPresence, PresenceLayer},
    structure::Camp,
};
use bevy::{prelude::*, reflect::TypePath};
use leafwing_input_manager::prelude::*;
use smallvec::SmallVec;

#[derive(Actionlike, TypePath, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    BreakCamp,
    Camp,
    Cancel,
    CollectCrystals,
    CreateParty,
    Deselect,
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
    SelectNext,
    SplitParty,
    ToggleMainMenu,
    ToggleInspector,
    ZoomCamera,
    Save,
}

pub fn magic_cancel(
    mut action_state: ResMut<ActionState<Action>>,
    menu_layer_query: Query<&Visibility, With<MenuLayer>>,
    selection_query: Query<&Selection>,
) {
    let actiondata = action_state.action_data(Action::Cancel).clone();

    // Close menu
    if let Ok(menu_layer_visibility) = menu_layer_query.get_single() {
        if *menu_layer_visibility == Visibility::Inherited {
            action_state.set_action_data(Action::ToggleMainMenu, actiondata);
            return;
        }
    }

    // Deselect
    if selection_query.iter().any(|s| s.is_selected) {
        action_state.set_action_data(Action::Deselect, actiondata);
        return;
    }

    // Open menu
    action_state.set_action_data(Action::ToggleMainMenu, actiondata);
}

pub fn handle_deselect(mut selection_query: Query<&mut Selection>) {
    for mut selection in selection_query.iter_mut() {
        if selection.is_selected {
            selection.is_selected = false;
        }
    }
}

pub fn handle_select_next(
    mut commands: Commands,
    mut selection_param_set: ParamSet<(
        NextSelectionQuery,
        Query<(Entity, &mut Selection, &MapPresence)>,
    )>,
    camera_query: Query<Entity, With<CameraControl>>,
) {
    let camera_entity = camera_query.single();
    let Some(next) = selection_param_set.p0().get() else {
        return;
    };
    for (entity, mut selection, presence) in &mut selection_param_set.p1() {
        if entity == next {
            selection.is_selected = true;
            commands
                .entity(camera_entity)
                .insert(CameraTarget::from_hexcoord(presence.position));
        } else if selection.is_selected {
            selection.is_selected = false;
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
            game_action_queue.add(GameAction::EnterCamp(entity, camp_entity));
        } else {
            game_action_queue.add(GameAction::MakeCamp(entity));
        }
    }
}

pub fn handle_break_camp(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, _) in party_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::BreakCamp(entity));
    }
}

pub fn handle_create_party(
    camp_query: Query<(Entity, &Group, &Selection), With<Camp>>,
    character_query: Query<(Entity, &Selection), With<Character>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, group, _) in camp_query.iter().filter(|(_, _, s)| s.is_selected) {
        let selected: SmallVec<[Entity; 8]> = character_query
            .iter_many(&group.members)
            .filter(|(_, s)| s.is_selected)
            .map(|(e, _)| e)
            .collect();
        if !selected.is_empty() {
            game_action_queue.add(GameAction::CreatePartyFromCamp(entity, selected));
        }
    }
}

pub fn handle_split_party(
    party_query: Query<(Entity, &Group, &Selection), With<Party>>,
    character_query: Query<(Entity, &Selection), With<Character>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, group, _) in party_query.iter().filter(|(_, _, s)| s.is_selected) {
        let selected: SmallVec<[Entity; 8]> = character_query
            .iter_many(&group.members)
            .filter(|(_, s)| s.is_selected)
            .map(|(e, _)| e)
            .collect();
        if !selected.is_empty() {
            game_action_queue.add(GameAction::SplitParty(entity, selected));
        }
    }
}

pub fn handle_merge_party(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    let selected_parties: SmallVec<[Entity; 8]> = party_query
        .iter()
        .filter(|(_, s)| s.is_selected)
        .map(|(e, _)| e)
        .collect();
    if !selected_parties.is_empty() {
        game_action_queue.add(GameAction::MergeParty(selected_parties));
    }
}

pub fn handle_collect_crystals(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (party, _) in party_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::CollectCrystals(party));
    }
}

pub fn handle_open_portal(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (party, _) in party_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::OpenPortal(party));
    }
}

pub fn handle_resume_move(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, _) in party_query.iter().filter(|(_, s)| s.is_selected) {
        game_action_queue.add(GameAction::ResumeMove(entity));
    }
}
