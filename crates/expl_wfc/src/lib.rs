pub mod cell;
mod error;
pub mod generator;
pub mod seed;
pub mod template;
pub mod tile;
pub mod util;

pub use error::WFCError;
pub use generator::Generator;
pub use seed::{Seed, SeedType};
pub use template::Template;

pub type TileId = usize;
