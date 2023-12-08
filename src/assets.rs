use crate::{
    map_generator::MapTemplate,
    terrain::{Codex, Decoration, Terrain},
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum AssetState {
    #[default]
    Loading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
pub struct MainAssets {
    #[asset(path = "textures/cloud.png")]
    pub cloud_texture: Handle<Image>,
    #[asset(path = "models/indicator.obj")]
    pub indicator_mesh: Handle<Mesh>,
    #[asset(path = "models/tent.obj")]
    pub tent_mesh: Handle<Mesh>,
    #[asset(path = "models/blob.obj")]
    pub blob_mesh: Handle<Mesh>,
    #[asset(path = "models/portal.obj")]
    pub portal_mesh: Handle<Mesh>,
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
