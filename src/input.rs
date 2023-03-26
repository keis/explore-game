#![allow(clippy::type_complexity)]

use crate::{
    action::{GameAction, GameActionQueue},
    camera::{CameraControl, CameraTarget},
    character::Movement,
    interface::MenuLayer,
    map::{MapPosition, MapPresence, PathGuided, Zone},
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
            .add_systems((handle_deselect, handle_select_next))
            .add_system(
                magic_cancel
                    .after(InputManagerSystem::ManualControl)
                    .in_base_set(CoreSet::PreUpdate),
            )
            .add_system(handle_picking_events.in_base_set(CoreSet::PostUpdate));
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
            if *menu_layer_visibility == Visibility::Inherited {
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

fn _find_next(
    party_query: &Query<(Entity, &mut Selection, &MapPresence, Option<&Movement>)>,
) -> Option<Entity> {
    let mut selected = None;
    for (entity, selection, _, m) in party_query.iter() {
        if selection.selected() {
            selected = Some(entity);
        } else if let Some(movement) = m {
            if selected.is_some() && movement.points > 0 {
                return Some(entity);
            }
        }
    }

    for (entity, _, _, m) in party_query.iter() {
        if selected == Some(entity) {
            break;
        }
        if let Some(movement) = m {
            if movement.points > 0 {
                if selected == Some(entity) {
                    return None;
                }
                return Some(entity);
            }
        }
    }

    None
}

fn handle_select_next(
    mut commands: Commands,
    action_state_query: Query<&ActionState<Action>>,
    mut party_query: Query<(Entity, &mut Selection, &MapPresence, Option<&Movement>)>,
    camera_query: Query<Entity, With<CameraControl>>,
) {
    let action_state = action_state_query.single();
    let camera_entity = camera_query.single();
    if action_state.just_pressed(Action::SelectNext) {
        let Some(next) = _find_next(&party_query) else { return };
        for (entity, mut selection, presence, _) in party_query.iter_mut() {
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
