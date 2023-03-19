use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct MainAssets {
    #[asset(path = "textures/cloud.png")]
    pub cloud_texture: Handle<Image>,
    #[asset(path = "textures/mountains_inner.png")]
    pub mountain_texture: Handle<Image>,
    #[asset(path = "textures/ocean_inner.png")]
    pub ocean_texture: Handle<Image>,
    #[asset(path = "textures/conifer_forest_inner.png")]
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
}
