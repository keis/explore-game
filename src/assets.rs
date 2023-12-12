use crate::{
    actor::Creature,
    map_generator::MapTemplate,
    structure::Structure,
    terrain::{Decoration, Terrain},
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use expl_codex::Codex;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum AssetState {
    #[default]
    Loading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
pub struct MainAssets {
    #[asset(path = "icons/swords-emblem.png")]
    pub swords_emblem_icon: Handle<Image>,
    #[asset(path = "maps/default.template.txt")]
    pub map_template: Handle<MapTemplate>,
}

#[derive(AssetCollection, Resource)]
pub struct CodexAssets {
    #[asset(path = "codex/default.terrain.toml")]
    pub terrain_codex: Handle<Codex<Terrain>>,
    #[asset(path = "codex/default.decoration.toml")]
    pub decoration_codex: Handle<Codex<Decoration>>,
    #[asset(path = "codex/default.structure.toml")]
    pub structure_codex: Handle<Codex<Structure>>,
    #[asset(path = "codex/default.creature.toml")]
    pub creature_codex: Handle<Codex<Creature>>,
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AssetState>()
            .add_loading_state(
                LoadingState::new(AssetState::Loading).continue_to_state(AssetState::Loaded),
            )
            .add_collection_to_loading_state::<_, MainAssets>(AssetState::Loading)
            .add_collection_to_loading_state::<_, CodexAssets>(AssetState::Loading);
    }
}
