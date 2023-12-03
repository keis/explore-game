use crate::{
    action::ActionPlugin,
    actor::ActorPlugin,
    assets::AssetsPlugin,
    camera::CameraControlPlugin,
    combat::CombatPlugin,
    enemy::EnemyPlugin,
    floating_text::FloatingTextPlugin,
    input::InputPlugin,
    inspector::InspectorPlugin,
    interface::InterfacePlugin,
    inventory::InventoryPlugin,
    map::MapPlugin,
    map_generator::MapGeneratorPlugin,
    path::PathPlugin,
    scene::{ScenePlugin, SceneSet, SceneState},
    structure::StructurePlugin,
    terrain::TerrainPlugin,
    turn::TurnPlugin,
};
use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    prelude::*,
};

pub struct ExplPlugins;

impl PluginGroup for ExplPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy_mod_billboard::prelude::BillboardPlugin)
            .add(bevy_mod_outline::OutlinePlugin)
            .add(bevy_obj::ObjPlugin)
            .add(bevy_sprite3d::Sprite3dPlugin)
            .add(noisy_bevy::NoisyShaderPlugin)
            .add(ActionPlugin)
            .add(ActorPlugin)
            .add(AssetsPlugin)
            .add(CameraControlPlugin)
            .add(CombatPlugin)
            .add(EnemyPlugin)
            .add(FloatingTextPlugin)
            .add(InputPlugin)
            .add(InspectorPlugin)
            .add(InterfacePlugin)
            .add(InventoryPlugin)
            .add(MapGeneratorPlugin)
            .add(MapPlugin {
                setup_schedule: OnEnter(SceneState::Active),
                setup_set: SceneSet::Populate,
            })
            .add(PathPlugin)
            .add(ScenePlugin)
            .add(StructurePlugin)
            .add(TerrainPlugin)
            .add(TurnPlugin)
    }
}
