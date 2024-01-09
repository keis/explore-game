use super::{action::*, component::*, event::*, system::*};
use crate::{scene::SceneState, turn};
use bevy::prelude::*;
use bevy_mod_picking::{
    backend::prelude::PickSet,
    highlight::update_highlight_assets,
    prelude::{Click, DebugPickingPlugin, DefaultPickingPlugins, Out, Over, Pointer},
};
use leafwing_input_manager::{
    common_conditions::action_just_pressed, plugin::InputManagerSystem, prelude::*,
};

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
                handle_deselect.run_if(action_just_pressed(Action::Deselect)),
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
            magic_cancel
                .in_set(InputManagerSystem::ManualControl)
                .after(InputManagerSystem::Update)
                .run_if(action_just_pressed(Action::Cancel)),
        )
        .add_systems(
            PreUpdate,
            (
                (
                    handle_pointer_click_events.run_if(on_event::<Pointer<Click>>()),
                    handle_pointer_over_events.run_if(on_event::<Pointer<Over>>()),
                    handle_pointer_out_events.run_if(on_event::<Pointer<Out>>()),
                )
                    .in_set(PickSet::Last),
                (
                    apply_zone_activated_events.map(bevy::utils::warn),
                    apply_selection_events
                        .run_if(on_event::<Select>().or_else(on_event::<Deselect>())),
                )
                    .after(handle_pointer_click_events),
                update_selection_highlight
                    .after(update_highlight_assets::<StandardMaterial>)
                    .after(apply_selection_events),
                update_interaction_highlight
                    .after(update_highlight_assets::<StandardMaterial>)
                    .after(apply_selection_events),
            )
                .run_if(in_state(SceneState::Active)),
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
