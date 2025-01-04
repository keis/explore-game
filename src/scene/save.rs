use crate::{action, actor, creature, input, inventory, structure, terrain, turn};
use bevy::prelude::*;
use expl_map;
use moonshine_save::save::SaveInput;
use platform_dirs::AppDirs;
use std::path::PathBuf;

pub use moonshine_save::prelude::{save_with, LoadSystem, Save};

#[derive(Resource)]
pub struct Loaded;

pub fn maybe_mark_as_loaded(world: &mut World) {
    if world.query::<&expl_map::MapLayout>().iter(world).len() != 0 {
        world.insert_resource(Loaded);
    }
}

pub fn save_location() -> PathBuf {
    AppDirs::new(Some("explore-game"), true)
        .map(|appdirs| appdirs.data_dir.join("save-state.ron"))
        .unwrap()
}

pub fn filter_with_enabled_components() -> SaveInput {
    let components = SceneFilter::deny_all()
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
        .allow::<Transform>();

    let resources = SceneFilter::deny_all().allow::<turn::Turn>();

    SaveInput {
        components,
        resources,
        ..default()
    }
}
