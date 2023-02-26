use crate::State;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

mod assets;
mod camp;
mod character;
mod color;
mod menu;
mod party;
mod shell;
mod tooltip;

pub use assets::InterfaceAssets;
pub use menu::MenuLayer;

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(State::AssetLoading)
                .continue_to_state(State::Running)
                .with_collection::<InterfaceAssets>(),
        )
        .add_system_set(
            SystemSet::on_enter(State::Running)
                .with_system(shell::spawn_shell)
                .with_system(menu::spawn_menu),
        )
        .add_system(party::update_party_list.with_run_criteria(party::run_if_any_party_changed))
        .add_system(camp::update_camp_list.with_run_criteria(camp::run_if_any_camp_changed))
        .add_system(
            character::update_character_list
                .with_run_criteria(character::run_if_any_party_or_selection_changed),
        )
        .add_system_set(
            SystemSet::on_update(State::Running)
                .with_system(party::update_party_selection)
                .with_system(party::update_party_movement_points)
                .with_system(party::update_party_size)
                .with_system(party::handle_party_display_interaction)
                .with_system(camp::update_camp_selection)
                .with_system(camp::handle_camp_display_interaction)
                .with_system(character::update_character_selection)
                .with_system(character::update_character_health)
                .with_system(character::handle_character_display_interaction)
                .with_system(shell::update_turn_text)
                .with_system(shell::update_zone_text)
                .with_system(shell::handle_move_button_interaction)
                .with_system(shell::handle_turn_button_interaction)
                .with_system(shell::handle_camp_button_interaction)
                .with_system(shell::handle_break_camp_button_interaction)
                .with_system(shell::handle_create_party_button_interaction)
                .with_system(shell::handle_split_party_button_interaction)
                .with_system(shell::handle_merge_party_button_interaction)
                .with_system(tooltip::show_tooltip_on_hover)
                .with_system(menu::handle_toggle_main_menu)
                .with_system(menu::handle_save)
                .with_system(menu::handle_quit),
        );
    }
}
