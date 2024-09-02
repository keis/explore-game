mod assets;
mod camp;
mod character;
mod color;
mod game_over;
mod menu;
mod party;
mod plugin;
mod resource;
mod root;
mod selected;
mod shell;
mod stat;
mod styles;
mod widget;

pub use assets::InterfaceAssets;
pub use plugin::{InterfacePlugin, InterfaceState};

const DEFAULT_FONT: &str = "fonts/FiraMono-Medium.ttf";

mod prelude {
    pub use bevy::{color::palettes::css, prelude::*};
    pub use bevy_mod_picking::prelude::*;
    pub use bevy_mod_stylebuilder::*;
    pub use bevy_quill_core::prelude::*;
}
