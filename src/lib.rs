pub mod action;
pub mod assets;
pub mod camera;
pub mod camp;
pub mod character;
pub mod hexgrid;
pub mod indicator;
pub mod input;
pub mod interface;
pub mod map;
pub mod material;
pub mod party;
pub mod path;
pub mod slide;
pub mod turn;
pub mod wfc;

pub const VIEW_RADIUS: u32 = 2;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum State {
    AssetLoading,
    Running,
}
