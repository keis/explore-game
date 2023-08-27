use crate::{
    input::{action_just_pressed, Action, InputManagerSystem},
    State,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

mod assets;
mod camp;
mod character;
mod color;
mod databinding;
mod menu;
mod party;
mod shell;
mod stat;
mod tooltip;

pub use assets::InterfaceAssets;
pub use menu::MenuLayer;

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(State::AssetLoading).continue_to_state(State::Running),
        )
        .add_collection_to_loading_state::<_, InterfaceAssets>(State::AssetLoading)
        .add_systems(
            OnEnter(State::Running),
            (shell::spawn_shell, menu::spawn_menu),
        )
        .add_systems(
            Update,
            (
                party::update_party_list.run_if(party::run_if_any_party_changed),
                camp::update_camp_list.run_if(camp::run_if_any_camp_changed),
                character::update_character_list
                    .run_if(character::run_if_any_party_or_selection_changed),
            ),
        )
        .add_systems(
            Update,
            (
                party::update_party_selection,
                party::update_party_movement_points,
                party::update_party_size,
                party::update_party_crystals,
                party::handle_party_display_interaction,
                camp::update_camp_selection,
                camp::update_camp_crystals,
                camp::handle_camp_display_interaction,
                character::update_character_selection,
                character::update_character_health,
                character::handle_character_display_interaction,
            )
                .run_if(in_state(State::Running)),
        )
        .add_systems(
            Update,
            (
                shell::update_turn_text,
                shell::update_zone_text,
                tooltip::show_tooltip_on_hover,
            )
                .run_if(in_state(State::Running)),
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
                menu::handle_toggle_main_menu.run_if(action_just_pressed(Action::ToggleMainMenu)),
                menu::handle_save,
                menu::handle_quit,
            )
                .after(InputManagerSystem::ManualControl)
                .run_if(in_state(State::Running)),
        );
    }
}
