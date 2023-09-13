pub mod action;
pub mod actor;
pub mod assets;
pub mod camera;
pub mod cleanup;
pub mod combat;
pub mod crystals;
mod error;
pub mod input;
pub mod inspector;
pub mod interface;
pub mod map;
pub mod material;
pub mod path;
pub mod scene;
pub mod structure;
pub mod turn;

pub use error::ExplError;

pub const VIEW_RADIUS: u32 = 3;
