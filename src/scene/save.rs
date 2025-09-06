use crate::{action, actor, creature, input, inventory, structure, terrain, turn};
use bevy::prelude::*;
use expl_map;
use moonshine_save::{
    load::{LoadWorld, TriggerLoad},
    save::{SaveWorld, TriggerSave},
};
use platform_dirs::AppDirs;
use std::path::PathBuf;

pub use moonshine_save::save::Save;

#[derive(Resource)]
pub struct Loaded;

pub fn load_saved_scene(mut commands: Commands) {
    commands.trigger_load(LoadWorld::default_from_file(save_location()))
}

pub fn maybe_mark_as_loaded(world: &mut World) {
    if world.query::<&expl_map::MapLayout>().iter(world).len() != 0 {
        world.insert_resource(Loaded);
    }
}

#[allow(deprecated)]
pub fn handle_save(mut commands: Commands) {
    let mut save_world = SaveWorld::<With<Save>>::into_file(save_location());
    save_world.input.components = component_filter();
    save_world.input.resources = resource_filter();
    commands.trigger_save(save_world);
}

pub fn save_location() -> PathBuf {
    AppDirs::new(Some("explore-game"), true)
        .map(|appdirs| appdirs.data_dir.join("save-state.ron"))
        .unwrap()
}

pub fn component_filter() -> SceneFilter {
    SceneFilter::deny_all()
        .allow::<Name>()
        .allow::<Save>()
        .allow::<action::ActionPoints>()
        .allow::<actor::ActorId>()
        .allow::<actor::Character>()
        .allow::<actor::Enemy>()
        .allow::<actor::Group>()
        .allow::<actor::Members>()
        .allow::<actor::Party>()
        .allow::<actor::Slide>()
        .allow::<creature::Attack>()
        .allow::<creature::Corpse>()
        .allow::<creature::CreatureId>()
        .allow::<creature::Health>()
        .allow::<input::Selection>()
        .allow::<inventory::Inventory>()
        .allow::<expl_map::Fog>()
        .allow::<expl_map::FogRevealer>()
        .allow::<expl_map::MapLayout>()
        .allow::<expl_map::MapPosition>()
        .allow::<expl_map::MapPresence>()
        .allow::<expl_map::ViewRadius>()
        .allow::<structure::Camp>()
        .allow::<structure::Portal>()
        .allow::<structure::SafeHaven>()
        .allow::<structure::Spawner>()
        .allow::<structure::StructureId>()
        .allow::<terrain::CrystalDeposit>()
        .allow::<terrain::TerrainId>()
        .allow::<terrain::ZoneDecorations>()
        .allow::<Transform>()
}

pub fn resource_filter() -> SceneFilter {
    SceneFilter::deny_all().allow::<turn::Turn>()
}
