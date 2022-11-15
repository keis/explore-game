use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct MainAssets {
    #[asset(path = "textures/cloud.png")]
    pub cloud_texture: Handle<Image>,
    #[asset(path = "textures/grass.png")]
    pub grass_texture: Handle<Image>,
    #[asset(path = "textures/lava.png")]
    pub lava_texture: Handle<Image>,
    #[asset(path = "models/indicator.stl")]
    pub indicator_mesh: Handle<Mesh>,
    #[asset(path = "models/tent.stl")]
    pub tent_mesh: Handle<Mesh>,
}
