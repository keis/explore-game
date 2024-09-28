use bevy::app::{PluginGroup, PluginGroupBuilder};

mod codex_buffer;
mod decoration_material;
mod portal_material;
mod structure_material;
mod water_material;
mod zone_material;

pub use decoration_material::{DecorationBuffer, DecorationMaterial, DecorationMaterialPlugin};
pub use portal_material::{PortalMaterial, PortalMaterialPlugin};
pub use structure_material::{StructureBuffer, StructureMaterial, StructureMaterialPlugin};
pub use water_material::{WaterMaterial, WaterMaterialPlugin};
pub use zone_material::{TerrainBuffer, ZoneMaterial, ZoneMaterialPlugin};

pub struct MaterialPlugins;

impl PluginGroup for MaterialPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(DecorationMaterialPlugin)
            .add(WaterMaterialPlugin)
            .add(ZoneMaterialPlugin)
            .add(PortalMaterialPlugin)
            .add(StructureMaterialPlugin)
    }
}
