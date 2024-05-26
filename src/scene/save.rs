use crate::{actor, creature, input, inventory, map, structure, terrain, turn};
use bevy::prelude::*;
use moonshine_save::save::SaveFilter;
use platform_dirs::AppDirs;
use std::path::PathBuf;

pub use moonshine_save::prelude::{save_with, LoadSystem, Save};

#[derive(Resource)]
pub struct Loaded;

pub fn maybe_mark_as_loaded(world: &mut World) {
    if world.query::<&map::MapLayout>().iter(world).len() != 0 {
        world.insert_resource(Loaded);
    }
}

pub fn save_location() -> PathBuf {
    AppDirs::new(Some("explore-game"), true)
        .map(|appdirs| appdirs.data_dir.join("save-state.ron"))
        .unwrap()
}

pub fn filter_with_enabled_components() -> SaveFilter {
    let components = SceneFilter::deny_all()
        .allow::<Name>()
        .allow::<Save>()
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
        .allow::<creature::Movement>()
        .allow::<input::Selection>()
        .allow::<inventory::Inventory>()
        .allow::<map::Fog>()
        .allow::<map::FogRevealer>()
        .allow::<map::MapLayout>()
        .allow::<map::MapPosition>()
        .allow::<map::MapPresence>()
        .allow::<map::ViewRadius>()
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

    SaveFilter {
        components,
        resources,
        ..default()
    }
}
