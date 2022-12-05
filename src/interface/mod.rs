use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

mod color;
mod menu;
mod shell;

pub use menu::MenuLayer;

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(shell::spawn_shell)
            .add_startup_system(menu::spawn_menu)
            .init_collection::<InterfaceAssets>()
            .add_system(shell::update_party_list)
            .add_system(shell::update_party_selection)
            .add_system(shell::update_party_movement_points)
            .add_system(shell::update_character_list)
            .add_system(shell::update_turn_text)
            .add_system(shell::update_zone_text)
            .add_system(shell::handle_party_display_interaction)
            .add_system(shell::handle_move_button_interaction)
            .add_system(shell::handle_turn_button_interaction)
            .add_system(shell::handle_camp_button_interaction)
            .add_system(shell::handle_break_camp_button_interaction)
            .add_system(menu::handle_toggle_main_menu)
            .add_system(menu::handle_save)
            .add_system(menu::handle_quit);
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
