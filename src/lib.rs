pub mod action;
pub mod assets;
pub mod camera;
pub mod character;
pub mod combat;
pub mod crystals;
pub mod enemy;
mod error;
pub mod input;
pub mod inspector;
pub mod interface;
pub mod map;
pub mod material;
pub mod party;
pub mod path;
pub mod scene;
pub mod slide;
pub mod structure;
pub mod turn;

pub use error::ExplError;

pub const VIEW_RADIUS: u32 = 3;
