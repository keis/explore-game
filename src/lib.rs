pub mod action;
pub mod assets;
pub mod camera;
pub mod camp;
pub mod character;
pub mod combat;
pub mod crystals;
pub mod enemy;
pub mod indicator;
pub mod input;
pub mod interface;
pub mod light;
pub mod map;
pub mod material;
pub mod party;
pub mod path;
pub mod scene;
pub mod selection;
pub mod slide;
pub mod turn;

use bevy::prelude::*;

pub const VIEW_RADIUS: u32 = 3;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum State {
    #[default]
    AssetLoading,
    Running,
}
