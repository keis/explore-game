#![allow(clippy::type_complexity)]

use crate::{
    action::{GameAction, GameActionQueue},
    camera::{CameraControl, CameraTarget},
    character::Character,
    interface::MenuLayer,
    map::{GameMap, MapPosition, MapPresence, PathGuided, Zone},
    party::{Group, Party},
    selection::NextSelectionQuery,
    structure::Camp,
    turn::Turn,
    State,
};
use bevy::prelude::*;
use bevy_mod_picking::{DefaultPickingPlugins, PickingEvent, Selection};
use smallvec::SmallVec;

pub use leafwing_input_manager::{
    common_conditions::action_just_pressed, plugin::InputManagerSystem, prelude::*,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPickingPlugins)
            .add_plugin(InputManagerPlugin::<Action>::default())
            .init_resource::<ActionState<Action>>()
            .insert_resource(input_map())
            .add_systems(
                (
                    handle_deselect.run_if(action_just_pressed(Action::Deselect)),
                    handle_select_next.run_if(action_just_pressed(Action::SelectNext)),
                    handle_resume_move.run_if(action_just_pressed(Action::ResumeMove)),
                    handle_camp.run_if(action_just_pressed(Action::Camp)),
                    handle_break_camp.run_if(action_just_pressed(Action::BreakCamp)),
                    handle_create_party.run_if(action_just_pressed(Action::CreateParty)),
                    handle_split_party.run_if(action_just_pressed(Action::SplitParty)),
                    handle_merge_party.run_if(action_just_pressed(Action::MergeParty)),
                    handle_collect_crystals.run_if(action_just_pressed(Action::CollectCrystals)),
                    handle_next_turn.run_if(action_just_pressed(Action::NextTurn)),
                )
                    .after(InputManagerSystem::ManualControl)
                    .in_set(OnUpdate(State::Running)),
            )
            .add_system(
                magic_cancel
                    .in_set(InputManagerSystem::ManualControl)
                    .run_if(action_just_pressed(Action::Cancel)),
            )
            .add_system(handle_picking_events.in_base_set(CoreSet::PostUpdate));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
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
    ZoomCamera,
}

fn input_map() -> InputMap<Action> {
    InputMap::default()
        .insert(KeyCode::Up, Action::PanCameraUp)
        .insert(KeyCode::Down, Action::PanCameraDown)
        .insert(KeyCode::Left, Action::PanCameraLeft)
        .insert(KeyCode::Right, Action::PanCameraRight)
        .insert(KeyCode::LControl, Action::MultiSelect)
        .insert(KeyCode::Escape, Action::Cancel)
        .insert(KeyCode::Space, Action::SelectNext)
        .insert(KeyCode::M, Action::ResumeMove)
        .insert(KeyCode::C, Action::Camp)
        .insert(KeyCode::Return, Action::NextTurn)
        .insert(SingleAxis::mouse_wheel_y(), Action::ZoomCamera)
        .insert(MouseButton::Right, Action::PanCamera)
        .insert(DualAxis::mouse_motion(), Action::PanCameraMotion)
        .build()
}

fn magic_cancel(
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
    if selection_query.iter().any(|s| s.selected()) {
        action_state.set_action_data(Action::Deselect, actiondata);
        return;
    }

    // Open menu
    action_state.set_action_data(Action::ToggleMainMenu, actiondata);
}

fn handle_deselect(mut selection_query: Query<&mut Selection>) {
    for mut selection in selection_query.iter_mut() {
        if selection.selected() {
            selection.set_selected(false);
        }
    }
}

fn handle_select_next(
    mut commands: Commands,
    mut selection_param_set: ParamSet<(
        NextSelectionQuery,
        Query<(Entity, &mut Selection, &MapPresence)>,
    )>,
    camera_query: Query<Entity, With<CameraControl>>,
) {
    let camera_entity = camera_query.single();
    let Some(next) = selection_param_set.p0().get() else { return };
    for (entity, mut selection, presence) in &mut selection_param_set.p1() {
        if entity == next {
            selection.set_selected(true);
            commands
                .entity(camera_entity)
                .insert(CameraTarget::from_hexcoord(presence.position));
        } else if selection.selected() {
            selection.set_selected(false);
        }
    }
}

fn handle_picking_events(
    mut events: EventReader<PickingEvent>,
    zone_query: Query<&MapPosition, With<Zone>>,
    presence_query: Query<(Entity, &Selection), (With<PathGuided>, With<MapPresence>)>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for event in &mut events {
        let PickingEvent::Clicked(e) = event else { continue };
        let Ok(zone_position) = zone_query.get(*e) else { continue };
        for (entity, _) in presence_query.iter().filter(|(_, s)| s.selected()) {
            game_action_queue.add(GameAction::MoveTo(entity, zone_position.0));
        }
    }
}

pub fn handle_resume_move(
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, _) in party_query.iter().filter(|(_, s)| s.selected()) {
        game_action_queue.add(GameAction::ResumeMove(entity));
    }
}

pub fn handle_camp(
    party_query: Query<(Entity, &MapPresence, &Selection), With<Party>>,
    map_query: Query<&GameMap>,
    camp_query: Query<Entity, With<Camp>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    info!("handle camp");
    for (entity, presence, _) in party_query.iter().filter(|(_, _, s)| s.selected()) {
        let Ok(map) = map_query.get(presence.map) else { continue };
        if let Some(camp_entity) = camp_query.iter_many(map.presence(presence.position)).next() {
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
    for (entity, _) in party_query.iter().filter(|(_, s)| s.selected()) {
        game_action_queue.add(GameAction::BreakCamp(entity));
    }
}

pub fn handle_create_party(
    camp_query: Query<(Entity, &Group, &Selection), With<Camp>>,
    character_query: Query<(Entity, &Selection), With<Character>>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for (entity, group, _) in camp_query.iter().filter(|(_, _, s)| s.selected()) {
        let selected: SmallVec<[Entity; 8]> = character_query
            .iter_many(&group.members)
            .filter(|(_, s)| s.selected())
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
    for (entity, group, _) in party_query.iter().filter(|(_, _, s)| s.selected()) {
        let selected: SmallVec<[Entity; 8]> = character_query
            .iter_many(&group.members)
            .filter(|(_, s)| s.selected())
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
        .filter(|(_, s)| s.selected())
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
    for (party, _) in party_query.iter().filter(|(_, s)| s.selected()) {
        game_action_queue.add(GameAction::CollectCrystals(party));
    }
}

pub fn handle_next_turn(mut turn: ResMut<Turn>) {
    turn.number += 1;
}

#[cfg(test)]
mod tests {
    use super::{handle_select_next, Action, ActionState, CameraControl, MapPresence, Selection};
    use crate::{character::Movement, map::tests::spawn_game_map};
    use bevy::prelude::*;
    use rstest::*;

    #[fixture]
    pub fn app() -> App {
        let mut app = App::new();
        let map_entity = spawn_game_map(&mut app);
        app.insert_resource(ActionState::<Action>::default());
        app.world.spawn(CameraControl::default());
        app.world.spawn((
            MapPresence {
                map: map_entity,
                position: (1, 1).into(),
            },
            Selection::default(),
            Movement { points: 2 },
        ));
        app.world.spawn((
            MapPresence {
                map: map_entity,
                position: (2, 0).into(),
            },
            Selection::default(),
            Movement { points: 2 },
        ));
        app
    }

    pub fn get_selected_entities(app: &mut App) -> Vec<Entity> {
        app.world
            .query::<(Entity, &Selection)>()
            .iter(&app.world)
            .filter(|(_, s)| s.selected())
            .map(|(e, _)| e)
            .collect()
    }

    pub fn press_select_next(app: &mut App) {
        app.world
            .resource_mut::<ActionState<Action>>()
            .press(Action::SelectNext);
    }

    #[rstest]
    pub fn select_next(mut app: App) {
        app.add_system(handle_select_next);

        press_select_next(&mut app);
        app.update();

        let selected1 = get_selected_entities(&mut app);

        press_select_next(&mut app);
        app.update();

        let selected2 = get_selected_entities(&mut app);

        press_select_next(&mut app);
        app.update();

        let selected3 = get_selected_entities(&mut app);

        assert_eq!(selected1.len(), 1);
        assert_eq!(selected2.len(), 1);
        assert_eq!(selected3.len(), 1);
        assert_ne!(selected1[0], selected2[0]);
        assert_eq!(selected1[0], selected3[0]);
    }
}
