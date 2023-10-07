use crate::{actor, combat, input, map, structure, terrain, turn};
use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*};
use moonshine_save::save::{
    finish, into_file, save_scene, EntityFilter, SaveFilter, SavePipeline, SaveSet,
};
use platform_dirs::AppDirs;
use std::path::PathBuf;

pub use moonshine_save::prelude::{LoadSet, Save};

pub fn save_location() -> PathBuf {
    AppDirs::new(Some("explore-game"), true)
        .map(|appdirs| appdirs.data_dir.join("save-state.ron"))
        .unwrap()
}

pub fn save_into_file(intopath: impl Into<PathBuf>) -> SavePipeline {
    let path: PathBuf = intopath.into();
    let parent = path.parent().unwrap().to_path_buf();
    ensure_save_directory(parent)
        .pipe(warn)
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

    let mut components = SceneFilter::deny_all();
    components.allow::<Name>();
    components.allow::<Save>();
    components.allow::<actor::character::Character>();
    components.allow::<actor::character::Movement>();
    components.allow::<actor::enemy::Enemy>();
    components.allow::<actor::party::Group>();
    components.allow::<actor::party::GroupMember>();
    components.allow::<actor::party::Party>();
    components.allow::<actor::slide::Slide>();
    components.allow::<combat::Attack>();
    components.allow::<combat::Health>();
    components.allow::<input::Selection>();
    components.allow::<map::Fog>();
    components.allow::<map::MapLayout>();
    components.allow::<map::MapPosition>();
    components.allow::<map::MapPresence>();
    components.allow::<map::Offset>();
    components.allow::<map::ViewRadius>();
    components.allow::<structure::Camp>();
    components.allow::<structure::Portal>();
    components.allow::<structure::Spawner>();
    components.allow::<terrain::CrystalDeposit>();
    components.allow::<terrain::Height>();
    components.allow::<terrain::Terrain>();
    components.allow::<terrain::ZoneDecorations>();
    components.allow::<Transform>();

    let mut resources = SceneFilter::deny_all();
    resources.allow::<turn::Turn>();

    SaveFilter {
        entities,
        components,
        resources,
    }
}
