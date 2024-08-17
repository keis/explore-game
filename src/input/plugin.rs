use super::{action::*, component::*, resource::*, system::*};
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
        .init_resource::<SelectedIndex>()
        .observe(SelectedIndex::on_select)
        .observe(SelectedIndex::on_deselect)
        .insert_resource(input_map())
        .observe(apply_zone_activated_event)
        .observe(apply_select_event)
        .observe(apply_deselect_event)
        .observe(apply_selection_over_event)
        .observe(apply_selection_out_event)
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
                handle_pointer_click_events.run_if(on_event::<Pointer<Click>>()),
                handle_pointer_over_events.run_if(on_event::<Pointer<Over>>()),
                handle_pointer_out_events.run_if(on_event::<Pointer<Out>>()),
            )
                .in_set(InputSet::ProcessInput)
                .run_if(in_state(SceneState::Active)),
        );
    }
}

fn input_map() -> InputMap<Action> {
    InputMap::default()
        .with(Action::PanCameraUp, KeyCode::ArrowUp)
        .with(Action::PanCameraDown, KeyCode::ArrowDown)
        .with(Action::PanCameraLeft, KeyCode::ArrowLeft)
        .with(Action::PanCameraRight, KeyCode::ArrowRight)
        .with(Action::MultiSelect, KeyCode::ControlLeft)
        .with(Action::Cancel, KeyCode::Escape)
        .with(Action::SelectNext, KeyCode::Space)
        .with(Action::ResumeMove, KeyCode::KeyM)
        .with(Action::Camp, KeyCode::KeyC)
        .with(Action::NextTurn, KeyCode::Enter)
        .with(Action::PanCamera, MouseButton::Right)
        .with(Action::ToggleInspector, KeyCode::F12)
        .with_axis(Action::ZoomCamera, MouseScrollAxis::Y)
        .with_dual_axis(Action::PanCameraMotion, MouseMove::default())
}
