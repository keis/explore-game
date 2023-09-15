pub mod action;
pub mod actor;
pub mod assets;
pub mod camera;
pub mod cleanup;
pub mod combat;
mod error;
pub mod input;
pub mod inspector;
pub mod interface;
pub mod map;
pub mod map_generator;
pub mod material;
pub mod path;
pub mod scene;
pub mod structure;
pub mod terrain;
pub mod turn;

pub use error::ExplError;

pub const VIEW_RADIUS: u32 = 3;
