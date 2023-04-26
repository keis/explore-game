use bevy::prelude::*;

mod camp;
mod portal;

pub use camp::{Camp, CampBundle, CampParams};
pub use portal::{Portal, PortalBundle, PortalParams};

pub struct StructurePlugin;

impl Plugin for StructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camp::update_camp_view_radius);
    }
}
