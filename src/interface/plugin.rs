use super::{assets::InterfaceAssets, camp, character, game_over, menu, party, shell, tooltip};
use crate::{
    assets::AssetState,
    input::{action_just_pressed, Action, InputManagerSystem},
    scene::SceneState,
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
        .add_state::<InterfaceState>()
        .add_systems(
            OnEnter(AssetState::Loaded),
            (
                shell::spawn_shell,
                menu::spawn_menu,
                |mut next_state: ResMut<NextState<InterfaceState>>| {
                    next_state.set(InterfaceState::Menu)
                },
            ),
        )
        .add_systems(
            OnEnter(SceneState::Active),
            (
                party::update_party_list,
                camp::update_camp_list,
                character::update_character_list,
            ),
        )
        .add_systems(
            OnEnter(SceneState::GameOver),
            |mut next_state: ResMut<NextState<InterfaceState>>| {
                next_state.set(InterfaceState::GameOver)
            },
        )
        .add_systems(
            PreUpdate,
            shell::handle_action_button_interaction
                .in_set(InputManagerSystem::ManualControl)
                .after(InputManagerSystem::Update),
        )
        .add_systems(
            Update,
            (
                (
                    party::update_party_list.run_if(party::run_if_any_party_changed),
                    party::update_party_selection,
                    party::update_party_movement_points,
                    party::update_party_size,
                    party::update_party_crystals,
                    party::handle_party_display_interaction,
                ),
                (
                    camp::update_camp_list.run_if(camp::run_if_any_camp_changed),
                    camp::update_camp_selection,
                    camp::update_camp_crystals,
                    camp::handle_camp_display_interaction,
                ),
                (
                    character::update_character_list
                        .run_if(character::run_if_any_party_or_selection_changed),
                    character::update_character_selection,
                    character::update_character_health,
                    character::handle_character_display_interaction,
                ),
                shell::update_turn_text,
                shell::update_zone_text,
                tooltip::show_tooltip_on_hover,
            )
                .run_if(in_state(AssetState::Loaded)),
        )
        .add_systems(
            Update,
            (
                menu::handle_toggle_main_menu
                    .run_if(action_just_pressed(Action::ToggleMainMenu))
                    .run_if(in_state(SceneState::Active)),
                (
                    menu::handle_resume,
                    menu::handle_new_game,
                    menu::handle_save,
                    menu::handle_quit,
                    menu::menu_item_interaction_effect,
                )
                    .run_if(in_state(InterfaceState::Menu)),
                game_over::handle_new_game.run_if(in_state(InterfaceState::GameOver)),
            )
                .after(InputManagerSystem::ManualControl)
                .run_if(in_state(AssetState::Loaded)),
        )
        .add_systems(
            OnEnter(InterfaceState::Menu),
            (shell::hide_shell, menu::show_menu),
        )
        .add_systems(
            OnEnter(InterfaceState::Shell),
            (shell::show_shell, menu::hide_menu),
        )
        .add_systems(
            OnEnter(InterfaceState::GameOver),
            (
                shell::hide_shell,
                menu::hide_menu,
                game_over::spawn_game_over_screen,
            ),
        )
        .add_systems(
            OnExit(InterfaceState::GameOver),
            game_over::despawn_game_over_screen,
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
