mod assets;
mod color;
mod component;
mod plugin;
mod resource;
mod root;
mod styles;
mod view;
mod widget;

pub use assets::InterfaceAssets;
pub use plugin::{InterfacePlugin, InterfaceState};

const DEFAULT_FONT: &str = "fonts/FiraMono-Medium.ttf";

mod prelude {
    pub use bevy::{color::palettes::css, prelude::*};
    pub use bevy_mod_stylebuilder::*;
    pub use bevy_quill_core::prelude::*;
}
