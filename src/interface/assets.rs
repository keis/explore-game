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
    #[asset(path = "icons/gladius.png")]
    pub gladius_icon: Handle<Image>,
    #[asset(path = "icons/heart-shield.png")]
    pub heart_shield_icon: Handle<Image>,
    #[asset(path = "icons/minerals.png")]
    pub crystals_icon: Handle<Image>,
    #[asset(path = "icons/portal.png")]
    pub portal_icon: Handle<Image>,
    #[asset(path = "icons/magic-swirl.png")]
    pub magic_swirl_icon: Handle<Image>,
    #[asset(path = "icons/brutal-helm.png")]
    pub brutal_helm_icon: Handle<Image>,
}
