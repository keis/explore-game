use crate::{
    action::ActionPlugin, actor::ActorPlugin, assets::AssetsPlugin, camera::CameraControlPlugin,
    combat::CombatPlugin, creature::CreaturePlugin, enemy::EnemyPlugin,
    floating_text::FloatingTextPlugin, input::InputPlugin, inspector::InspectorPlugin,
    interface::InterfacePlugin, inventory::InventoryPlugin, map_generator::MapGeneratorPlugin,
    path::PathPlugin, scene::ScenePlugin, structure::StructurePlugin, terrain::TerrainPlugin,
    turn::TurnPlugin,
};
use bevy::app::{PluginGroup, PluginGroupBuilder};

pub struct ExplPlugins;

impl PluginGroup for ExplPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy_mod_billboard::prelude::BillboardPlugin)
            .add(bevy_mod_outline::OutlinePlugin)
            .add(bevy_obj::ObjPlugin)
            .add(bevy_quill_core::QuillPlugin)
            .add(bevy_sprite3d::Sprite3dPlugin)
            .add(expl_databinding::DataBindingPlugin)
            .add(expl_map::MapPlugin)
            .add(noisy_bevy::NoisyShaderPlugin)
            .add(ActionPlugin)
            .add(ActorPlugin)
            .add(AssetsPlugin)
            .add(CameraControlPlugin)
            .add(CombatPlugin)
            .add(CreaturePlugin)
            .add(EnemyPlugin)
            .add(FloatingTextPlugin)
            .add(InputPlugin)
            .add(InspectorPlugin)
            .add(InterfacePlugin)
            .add(InventoryPlugin)
            .add(MapGeneratorPlugin)
            .add(PathPlugin)
            .add(ScenePlugin)
            .add(StructurePlugin)
            .add(TerrainPlugin)
            .add(TurnPlugin)
    }
}
