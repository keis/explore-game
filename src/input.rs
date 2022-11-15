#![allow(clippy::type_complexity)]

use crate::action::{GameAction, GameActionQueue};
use crate::interface::MenuLayer;
use crate::map::{MapPosition, MapPresence, PathGuided};
use crate::zone::Zone;
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
            .insert(SingleAxis::mouse_wheel_y(), Action::ZoomCamera)
            .insert(MouseButton::Right, Action::PanCamera)
            .insert(DualAxis::mouse_motion(), Action::PanCameraMotion)
            .build(),
    });
}

pub fn magic_cancel(
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

pub fn handle_deselect(
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

pub fn handle_picking_events(
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
                info!("Selection event {:?}", e);
            }
            _ => {}
        }
    }
}
