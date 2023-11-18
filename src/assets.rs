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
    #[asset(path = "textures/forest.png")]
    pub forest_texture: Handle<Image>,
    #[asset(path = "models/indicator.obj")]
    pub indicator_mesh: Handle<Mesh>,
    #[asset(path = "models/tent.obj")]
    pub tent_mesh: Handle<Mesh>,
    #[asset(path = "models/blob.obj")]
    pub blob_mesh: Handle<Mesh>,
    #[asset(path = "models/pine.obj")]
    pub pine_mesh: Handle<Mesh>,
    #[asset(path = "models/crystals.obj")]
    pub crystals_mesh: Handle<Mesh>,
    #[asset(path = "models/portal.obj")]
    pub portal_mesh: Handle<Mesh>,
    #[asset(path = "icons/swords-emblem.png")]
    pub swords_emblem_icon: Handle<Image>,
}
