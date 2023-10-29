use super::{action::*, map::handle_zone_click_events, selection::*};
use crate::turn;
use bevy::prelude::*;
use bevy_mod_picking::{
    backend::prelude::PickSet,
    highlight::update_highlight_assets,
    prelude::{DebugPickingPlugin, DefaultPickingPlugins},
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
        .add_systems(Update, handle_zone_click_events)
        .add_event::<Select>()
        .add_event::<Deselect>()
        .add_systems(
            PreUpdate,
            (send_selection_events, apply_selection_events).in_set(PickSet::PostFocus),
        )
        .add_systems(
            PreUpdate,
            update_highlight
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
