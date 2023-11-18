use crate::{actor, combat, input, inventory, map, structure, terrain, turn};
use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*};
use moonshine_save::save::{
    finish, into_file, save_scene, EntityFilter, SaveFilter, SavePipeline, SaveSet,
};
use platform_dirs::AppDirs;
use std::path::PathBuf;

pub use moonshine_save::prelude::{LoadSet, Save};

#[derive(Resource)]
pub struct Loaded;

pub fn mark_as_loaded(world: &mut World) {
    world.insert_resource(Loaded);
}

pub fn save_location() -> PathBuf {
    AppDirs::new(Some("explore-game"), true)
        .map(|appdirs| appdirs.data_dir.join("save-state.ron"))
        .unwrap()
}

pub fn save_into_file(intopath: impl Into<PathBuf>) -> SavePipeline {
    let path: PathBuf = intopath.into();
    let parent = path.parent().unwrap().to_path_buf();
    ensure_save_directory(parent)
        .map(bevy::utils::warn)
        .pipe(filter_with_enabled_components::<With<Save>>)
        .pipe(save_scene)
        .pipe(into_file(path))
        .pipe(finish)
        .in_set(SaveSet::Save)
}

pub fn ensure_save_directory(parent: PathBuf) -> impl Fn() -> std::io::Result<()> {
    move || std::fs::create_dir_all(&parent)
}

pub fn filter_with_enabled_components<Filter: ReadOnlyWorldQuery>(
    entities: Query<Entity, Filter>,
) -> SaveFilter {
    let entities = EntityFilter::allow(&entities);

    let components = SceneFilter::deny_all()
        .allow::<Name>()
        .allow::<Save>()
        .allow::<actor::Character>()
        .allow::<actor::Enemy>()
        .allow::<actor::Group>()
        .allow::<actor::GroupMember>()
        .allow::<actor::Movement>()
        .allow::<actor::Party>()
        .allow::<actor::Slide>()
        .allow::<combat::Attack>()
        .allow::<combat::Health>()
        .allow::<input::Selection>()
        .allow::<inventory::Inventory>()
        .allow::<map::Fog>()
        .allow::<map::FogRevealer>()
        .allow::<map::MapLayout>()
        .allow::<map::MapPosition>()
        .allow::<map::MapPresence>()
        .allow::<map::Offset>()
        .allow::<map::ViewRadius>()
        .allow::<structure::Camp>()
        .allow::<structure::Portal>()
        .allow::<structure::SafeHaven>()
        .allow::<structure::Spawner>()
        .allow::<terrain::CrystalDeposit>()
        .allow::<terrain::Height>()
        .allow::<terrain::Terrain>()
        .allow::<terrain::ZoneDecorations>()
        .allow::<Transform>();

    let resources = SceneFilter::deny_all().allow::<turn::Turn>();

    SaveFilter {
        entities,
        components,
        resources,
    }
}
