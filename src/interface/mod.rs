use crate::State;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

mod character;
mod color;
mod menu;
mod party;
mod shell;

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
        .add_system_set(
            SystemSet::on_update(State::Running)
                .with_system(party::update_party_list)
                .with_system(party::update_party_selection)
                .with_system(party::update_party_movement_points)
                .with_system(party::handle_party_display_interaction)
                .with_system(character::update_character_list)
                .with_system(character::update_character_selection)
                .with_system(character::handle_character_display_interaction)
                .with_system(shell::update_turn_text)
                .with_system(shell::update_zone_text)
                .with_system(shell::handle_move_button_interaction)
                .with_system(shell::handle_turn_button_interaction)
                .with_system(shell::handle_camp_button_interaction)
                .with_system(shell::handle_break_camp_button_interaction)
                .with_system(menu::handle_toggle_main_menu)
                .with_system(menu::handle_save)
                .with_system(menu::handle_quit),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct InterfaceAssets {
    #[asset(path = "fonts/FiraMono-Medium.ttf")]
    font: Handle<Font>,
    #[asset(path = "icons/campfire.png")]
    campfire_icon: Handle<Image>,
    #[asset(path = "icons/knapsack.png")]
    knapsack_icon: Handle<Image>,
    #[asset(path = "icons/bottom-right-3d-arrow.png")]
    arrow_icon: Handle<Image>,
}
