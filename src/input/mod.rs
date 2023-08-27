#![allow(clippy::type_complexity)]

use crate::State;
use bevy::prelude::*;
use bevy_mod_picking::{
    backend::prelude::PickSet,
    highlight::update_highlight_assets,
    prelude::{DebugPickingPlugin, DefaultPickingPlugins},
};
use leafwing_input_manager::prelude::*;

mod action;
mod map;
mod selection;

pub use action::Action;
pub use leafwing_input_manager::{
    common_conditions::{action_just_pressed, action_toggle_active},
    plugin::InputManagerSystem,
    prelude::ActionState,
};
pub use selection::{Deselect, NextSelectionQuery, Select, Selection, SelectionBundle};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>()
                .disable::<bevy_mod_picking::prelude::SelectionPlugin>(),
        )
        .add_plugins(InputManagerPlugin::<Action>::default())
        .init_resource::<ActionState<Action>>()
        .insert_resource(input_map())
        .add_systems(
            Update,
            (
                action::handle_deselect.run_if(action_just_pressed(Action::Deselect)),
                action::handle_select_next.run_if(action_just_pressed(Action::SelectNext)),
                action::handle_resume_move.run_if(action_just_pressed(Action::ResumeMove)),
                action::handle_camp.run_if(action_just_pressed(Action::Camp)),
                action::handle_break_camp.run_if(action_just_pressed(Action::BreakCamp)),
                action::handle_create_party.run_if(action_just_pressed(Action::CreateParty)),
                action::handle_split_party.run_if(action_just_pressed(Action::SplitParty)),
                action::handle_merge_party.run_if(action_just_pressed(Action::MergeParty)),
                action::handle_collect_crystals
                    .run_if(action_just_pressed(Action::CollectCrystals)),
                action::handle_open_portal.run_if(action_just_pressed(Action::OpenPortal)),
                action::handle_next_turn.run_if(action_just_pressed(Action::NextTurn)),
            )
                .run_if(in_state(State::Running))
                .after(InputManagerSystem::ManualControl),
        )
        .add_systems(
            PreUpdate,
            action::magic_cancel
                .in_set(InputManagerSystem::ManualControl)
                .run_if(action_just_pressed(Action::Cancel)),
        )
        .add_systems(Update, map::handle_zone_click_events)
        .add_event::<Select>()
        .add_event::<Deselect>()
        .add_systems(
            PreUpdate,
            (
                selection::send_selection_events,
                selection::apply_selection_events,
            )
                .in_set(PickSet::PostFocus),
        )
        .add_systems(
            PreUpdate,
            selection::update_highlight
                .after(update_highlight_assets::<StandardMaterial>)
                .in_set(PickSet::Last),
        );
    }
}

fn input_map() -> InputMap<Action> {
    InputMap::default()
        .insert(KeyCode::Up, Action::PanCameraUp)
        .insert(KeyCode::Down, Action::PanCameraDown)
        .insert(KeyCode::Left, Action::PanCameraLeft)
        .insert(KeyCode::Right, Action::PanCameraRight)
        .insert(KeyCode::ControlLeft, Action::MultiSelect)
        .insert(KeyCode::Escape, Action::Cancel)
        .insert(KeyCode::Space, Action::SelectNext)
        .insert(KeyCode::M, Action::ResumeMove)
        .insert(KeyCode::C, Action::Camp)
        .insert(KeyCode::Return, Action::NextTurn)
        .insert(SingleAxis::mouse_wheel_y(), Action::ZoomCamera)
        .insert(MouseButton::Right, Action::PanCamera)
        .insert(DualAxis::mouse_motion(), Action::PanCameraMotion)
        .insert(KeyCode::F12, Action::ToggleInspector)
        .build()
}

#[cfg(test)]
mod tests {
    use super::{action::handle_select_next, Action, ActionState, Selection};
    use crate::{
        camera::CameraControl, character::Movement, map::tests::spawn_game_map, map::MapPresence,
    };
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
            .filter(|(_, s)| s.is_selected)
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
        app.add_systems(Update, handle_select_next);

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
