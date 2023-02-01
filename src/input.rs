#![allow(clippy::type_complexity)]

use crate::{
    action::{GameAction, GameActionQueue},
    camera::{CameraControl, CameraTarget},
    hex::coord_to_vec3,
    interface::MenuLayer,
    map::{MapPosition, MapPresence, PathGuided, Zone},
    party::Party,
};
use bevy::prelude::*;
use bevy_mod_picking::{DefaultPickingPlugins, PickingEvent, Selection};
use leafwing_input_manager::plugin::InputManagerSystem;
pub use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPickingPlugins)
            .add_plugin(InputManagerPlugin::<Action>::default())
            .add_startup_system(spawn_input_manager)
            .add_system(handle_deselect)
            .add_system(handle_select_next)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                magic_cancel.after(InputManagerSystem::ManualControl),
            )
            .add_system_to_stage(CoreStage::PostUpdate, handle_picking_events);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    PanCamera,
    PanCameraMotion,
    PanCameraUp,
    PanCameraDown,
    PanCameraLeft,
    PanCameraRight,
    ZoomCamera,
    MultiSelect,
    Cancel,
    ToggleMainMenu,
    Deselect,
    SelectNext,
}

fn spawn_input_manager(mut commands: Commands) {
    commands.spawn(InputManagerBundle {
        action_state: ActionState::default(),
        input_map: InputMap::default()
            .insert(KeyCode::Up, Action::PanCameraUp)
            .insert(KeyCode::Down, Action::PanCameraDown)
            .insert(KeyCode::Left, Action::PanCameraLeft)
            .insert(KeyCode::Right, Action::PanCameraRight)
            .insert(KeyCode::LControl, Action::MultiSelect)
            .insert(KeyCode::Escape, Action::Cancel)
            .insert(KeyCode::Space, Action::SelectNext)
            .insert(SingleAxis::mouse_wheel_y(), Action::ZoomCamera)
            .insert(MouseButton::Right, Action::PanCamera)
            .insert(DualAxis::mouse_motion(), Action::PanCameraMotion)
            .build(),
    });
}

fn magic_cancel(
    mut action_state_query: Query<&mut ActionState<Action>>,
    menu_layer_query: Query<&Visibility, With<MenuLayer>>,
    selection_query: Query<&Selection>,
) {
    let mut action_state = action_state_query.single_mut();
    let actiondata = action_state.action_data(Action::Cancel).clone();

    // Close menu
    if action_state.just_pressed(Action::Cancel) {
        if let Ok(menu_layer_visibility) = menu_layer_query.get_single() {
            if menu_layer_visibility.is_visible {
                action_state.set_action_data(Action::ToggleMainMenu, actiondata);
                return;
            }
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

fn handle_deselect(
    action_state_query: Query<&ActionState<Action>>,
    mut selection_query: Query<&mut Selection>,
) {
    let action_state = action_state_query.single();
    if action_state.just_pressed(Action::Deselect) {
        for mut selection in selection_query.iter_mut() {
            if selection.selected() {
                selection.set_selected(false);
            }
        }
    }
}

fn _find_selected_and_next(
    party_query: &Query<(Entity, &mut Selection, &MapPresence, &Party)>,
) -> (Option<Entity>, Option<Entity>) {
    let mut selected = None;
    for (entity, selection, _, party) in party_query.iter() {
        if selection.selected() {
            selected = Some(entity);
        } else if selected.is_some() && party.movement_points > 0 {
            return (selected, Some(entity));
        }
    }

    for (entity, _, _, party) in party_query.iter() {
        if party.movement_points > 0 {
            return (selected, Some(entity));
        }
    }

    (None, None)
}

fn handle_select_next(
    mut commands: Commands,
    action_state_query: Query<&ActionState<Action>>,
    mut party_query: Query<(Entity, &mut Selection, &MapPresence, &Party)>,
    camera_query: Query<Entity, With<CameraControl>>,
) {
    let action_state = action_state_query.single();
    let camera_entity = camera_query.single();
    if action_state.just_pressed(Action::SelectNext) {
        let (selected, next) = _find_selected_and_next(&party_query);
        if selected == next {
            return;
        }
        if let Some((_, mut selection, _, _)) = selected.and_then(|e| party_query.get_mut(e).ok()) {
            selection.set_selected(false);
        }
        if let Some((_, mut selection, presence, _)) =
            next.and_then(|e| party_query.get_mut(e).ok())
        {
            selection.set_selected(true);
            commands.entity(camera_entity).insert(CameraTarget {
                position: coord_to_vec3(presence.position) + Vec3::new(2.0, 20.0, 20.0),
            });
        }
    }
}

fn handle_picking_events(
    mut events: EventReader<PickingEvent>,
    zone_query: Query<&MapPosition, With<Zone>>,
    presence_query: Query<(Entity, &Selection), (With<PathGuided>, With<MapPresence>)>,
    mut game_action_queue: ResMut<GameActionQueue>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(e) => {
                if let Ok(zone_position) = zone_query.get(*e) {
                    info!("Clicked a zone: {:?}", zone_position);
                    for (entity, _) in presence_query.iter().filter(|(_, s)| s.selected()) {
                        game_action_queue.add(GameAction::MoveTo(entity, zone_position.0));
                    }
                } else {
                    info!("Clicked something: {:?}", e);
                }
            }
            PickingEvent::Selection(e) => {
                debug!("Selection event {:?}", e);
            }
            _ => {}
        }
    }
}
