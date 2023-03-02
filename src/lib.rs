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
pub mod map;
pub mod material;
pub mod party;
pub mod path;
pub mod slide;
pub mod turn;

pub const VIEW_RADIUS: u32 = 3;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum State {
    AssetLoading,
    Running,
}
