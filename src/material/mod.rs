use bevy::app::{PluginGroup, PluginGroupBuilder};

mod terrain_material;
mod water_material;
mod zone_material;

pub use terrain_material::{TerrainMaterial, TerrainMaterialPlugin};
pub use water_material::{WaterMaterial, WaterMaterialPlugin};
pub use zone_material::{ZoneMaterial, ZoneMaterialPlugin};

pub struct MaterialPlugins;

impl PluginGroup for MaterialPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TerrainMaterialPlugin)
            .add(WaterMaterialPlugin)
            .add(ZoneMaterialPlugin)
    }
}
