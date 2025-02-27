use super::{assets::InterfaceAssets, resource::*, root};
use crate::{
    actor::Party,
    assets::AssetState,
    input::{action_just_pressed, Action, InputManagerSystem},
    scene::SceneState,
    structure::Camp,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AssetState::Loading)
                .continue_to_state(AssetState::Loaded)
                .load_collection::<InterfaceAssets>(),
        )
        .init_state::<InterfaceState>()
        .add_sub_state::<ShellState>()
        .init_resource::<Index<Party>>()
        .init_resource::<Index<Camp>>()
        .add_observer(Index::<Party>::on_add)
        .add_observer(Index::<Party>::on_remove)
        .add_observer(Index::<Camp>::on_add)
        .add_observer(Index::<Camp>::on_remove)
        .add_systems(
            OnEnter(AssetState::Loaded),
            (
                root::spawn_interface_root,
                |mut next_state: ResMut<NextState<InterfaceState>>| {
                    next_state.set(InterfaceState::Menu)
                },
            ),
        )
        .add_systems(
            OnEnter(SceneState::GameOver),
            |mut next_state: ResMut<NextState<InterfaceState>>| {
                next_state.set(InterfaceState::GameOver)
            },
        )
        .add_systems(
            Update,
            handle_toggle_main_menu
                .run_if(action_just_pressed(Action::ToggleMainMenu))
                .run_if(in_state(SceneState::Active))
                .after(InputManagerSystem::ManualControl)
                .run_if(in_state(AssetState::Loaded)),
        );
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum InterfaceState {
    #[default]
    Hidden,
    Shell,
    Menu,
    GameOver,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, SubStates, Default)]
#[source(InterfaceState = InterfaceState::Shell)]
pub enum ShellState {
    #[default]
    Map,
    Camp {
        target: Entity,
    },
}

pub fn handle_toggle_main_menu(
    current_state: Res<State<InterfaceState>>,
    mut next_state: ResMut<NextState<InterfaceState>>,
) {
    next_state.set(match current_state.get() {
        InterfaceState::Hidden => InterfaceState::Menu,
        InterfaceState::Menu => InterfaceState::Shell,
        InterfaceState::Shell => InterfaceState::Menu,
        InterfaceState::GameOver => InterfaceState::GameOver,
    });
}
