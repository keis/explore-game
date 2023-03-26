use crate::State;
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
        .add_systems((shell::spawn_shell, menu::spawn_menu).in_schedule(OnEnter(State::Running)))
        .add_system(party::update_party_list.run_if(party::run_if_any_party_changed))
        .add_system(camp::update_camp_list.run_if(camp::run_if_any_camp_changed))
        .add_system(
            character::update_character_list
                .run_if(character::run_if_any_party_or_selection_changed),
        )
        .add_systems(
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
                .in_set(OnUpdate(State::Running)),
        )
        .add_systems(
            (
                shell::update_turn_text,
                shell::update_zone_text,
                shell::handle_move_button_interaction,
                shell::handle_turn_button_interaction,
                shell::handle_camp_button_interaction,
                shell::handle_break_camp_button_interaction,
                shell::handle_create_party_button_interaction,
                shell::handle_split_party_button_interaction,
                shell::handle_merge_party_button_interaction,
                shell::handle_collect_crystals_button_interaction,
                tooltip::show_tooltip_on_hover,
            )
                .in_set(OnUpdate(State::Running)),
        )
        .add_systems(
            (
                menu::handle_toggle_main_menu,
                menu::handle_save,
                menu::handle_quit,
            )
                .in_set(OnUpdate(State::Running)),
        );
    }
}
