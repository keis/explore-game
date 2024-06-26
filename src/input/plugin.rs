use super::{action::*, component::*, event::*, system::*};
use crate::{scene::SceneState, turn};
use bevy::prelude::*;
use bevy_mod_picking::{
    backend::prelude::PickSet,
    prelude::{Click, DebugPickingPlugin, DefaultPickingPlugins, Out, Over, Pointer},
};
use leafwing_input_manager::{
    common_conditions::action_just_pressed, plugin::InputManagerSystem, prelude::*,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum InputSet {
    ProcessInput,
    Selection,
    PostSelection,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_plugins(InputManagerPlugin::<Action>::default())
        .configure_sets(
            PreUpdate,
            (
                InputSet::ProcessInput,
                InputSet::Selection,
                InputSet::PostSelection,
            )
                .in_set(PickSet::Last)
                .chain(),
        )
        .register_type::<Selection>()
        .init_resource::<ActionState<Action>>()
        .add_event::<ZoneActivated>()
        .add_event::<ZoneOver>()
        .add_event::<ZoneOut>()
        .add_event::<Select>()
        .add_event::<Deselect>()
        .add_event::<SelectionOver>()
        .add_event::<SelectionOut>()
        .insert_resource(input_map())
        .add_systems(
            Update,
            (
                handle_enter_portal.run_if(action_just_pressed(Action::EnterPortal)),
                handle_select_next.run_if(action_just_pressed(Action::SelectNext)),
                handle_resume_move.run_if(action_just_pressed(Action::ResumeMove)),
                handle_camp.run_if(action_just_pressed(Action::Camp)),
                handle_break_camp.run_if(action_just_pressed(Action::BreakCamp)),
                handle_create_party.run_if(action_just_pressed(Action::CreateParty)),
                handle_split_party.run_if(action_just_pressed(Action::SplitParty)),
                handle_merge_party.run_if(action_just_pressed(Action::MergeParty)),
                handle_collect_crystals.run_if(action_just_pressed(Action::CollectCrystals)),
                handle_open_portal.run_if(action_just_pressed(Action::OpenPortal)),
                turn::set_system_turn.run_if(action_just_pressed(Action::NextTurn)),
            )
                .after(InputManagerSystem::ManualControl),
        )
        .add_systems(
            PreUpdate,
            (
                magic_cancel.run_if(action_just_pressed(Action::Cancel)),
                handle_deselect.run_if(action_just_pressed(Action::Deselect)),
            )
                .chain()
                .in_set(InputSet::ProcessInput)
                .in_set(InputManagerSystem::ManualControl),
        )
        .add_systems(
            PreUpdate,
            (
                (
                    handle_pointer_click_events.run_if(on_event::<Pointer<Click>>()),
                    handle_pointer_over_events.run_if(on_event::<Pointer<Over>>()),
                    handle_pointer_out_events.run_if(on_event::<Pointer<Out>>()),
                )
                    .in_set(InputSet::ProcessInput),
                (
                    apply_zone_activated_events.map(bevy::utils::warn),
                    apply_selection_events
                        .run_if(on_event::<Select>().or_else(on_event::<Deselect>())),
                )
                    .in_set(InputSet::Selection),
                apply_selection_over_out_events
                    .run_if(on_event::<SelectionOver>().or_else(on_event::<SelectionOut>()))
                    .in_set(InputSet::PostSelection),
            )
                .run_if(in_state(SceneState::Active)),
        );
    }
}

fn input_map() -> InputMap<Action> {
    InputMap::default()
        .insert(Action::PanCameraUp, KeyCode::ArrowUp)
        .insert(Action::PanCameraDown, KeyCode::ArrowDown)
        .insert(Action::PanCameraLeft, KeyCode::ArrowLeft)
        .insert(Action::PanCameraRight, KeyCode::ArrowRight)
        .insert(Action::MultiSelect, KeyCode::ControlLeft)
        .insert(Action::Cancel, KeyCode::Escape)
        .insert(Action::SelectNext, KeyCode::Space)
        .insert(Action::ResumeMove, KeyCode::KeyM)
        .insert(Action::Camp, KeyCode::KeyC)
        .insert(Action::NextTurn, KeyCode::Enter)
        .insert(Action::ZoomCamera, SingleAxis::mouse_wheel_y())
        .insert(Action::PanCamera, MouseButton::Right)
        .insert(Action::PanCameraMotion, DualAxis::mouse_motion())
        .insert(Action::ToggleInspector, KeyCode::F12)
        .build()
}
