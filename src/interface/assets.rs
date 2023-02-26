use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct InterfaceAssets {
    #[asset(path = "fonts/FiraMono-Medium.ttf")]
    pub font: Handle<Font>,
    #[asset(path = "icons/campfire.png")]
    pub campfire_icon: Handle<Image>,
    #[asset(path = "icons/knapsack.png")]
    pub knapsack_icon: Handle<Image>,
    #[asset(path = "icons/bottom-right-3d-arrow.png")]
    pub arrow_icon: Handle<Image>,
    #[asset(path = "icons/back-forth.png")]
    pub back_forth_icon: Handle<Image>,
    #[asset(path = "icons/cancel.png")]
    pub cancel_icon: Handle<Image>,
    #[asset(path = "icons/contract.png")]
    pub contract_icon: Handle<Image>,
    #[asset(path = "icons/footsteps.png")]
    pub footsteps_icon: Handle<Image>,
    #[asset(path = "icons/person.png")]
    pub person_icon: Handle<Image>,
}
