#![allow(clippy::type_complexity)]

use crate::{
    action::{GameAction, GameActionQueue},
    camera::{CameraControl, CameraTarget},
    interface::MenuLayer,
    map::{MapPosition, MapPresence, PathGuided, Zone},
    selection::NextSelectionQuery,
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

fn handle_select_next(
    mut commands: Commands,
    action_state_query: Query<&ActionState<Action>>,
    mut selection_param_set: ParamSet<(
        NextSelectionQuery,
        Query<(Entity, &mut Selection, &MapPresence)>,
    )>,
    camera_query: Query<Entity, With<CameraControl>>,
) {
    let action_state = action_state_query.single();
    let camera_entity = camera_query.single();
    if action_state.just_pressed(Action::SelectNext) {
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
        app.world.spawn(ActionState::<Action>::default());
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
            .query::<&mut ActionState<Action>>()
            .single_mut(&mut app.world)
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
