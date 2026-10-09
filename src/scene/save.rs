use crate::{action, actor, creature, input, inventory, structure, terrain, turn};
use bevy::prelude::*;
use expl_map;
use moonshine_save::{
    load::{LoadWorld, TriggerLoad},
    save::{SaveWorld, TriggerSave},
};
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Marks entities (and resources, via filter) to be saved.
///
/// Unlike moonshine's own marker, this one implements reflection so it
/// round-trips through save files. That keeps freshly spawned and loaded
/// entities indistinguishable for scene resets, which despawn `With<Save>`.
#[derive(Component, Reflect, Default, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Save;

#[derive(Resource)]
pub struct Loaded;

pub fn load_saved_scene(mut commands: Commands) {
    commands.trigger_load(LoadWorld::<With<Save>>::from_file(save_location()))
}

pub fn maybe_mark_as_loaded(world: &mut World) {
    if world.query::<&expl_map::MapLayout>().iter(world).len() != 0 {
        world.insert_resource(Loaded);
    }
}

#[allow(deprecated)]
pub fn handle_save(mut commands: Commands) {
    let mut save_world = SaveWorld::<With<Save>>::into_file(save_location());
    save_world.components = component_filter();
    save_world.resources = resource_filter();
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
        .allow::<actor::MemberOf>()
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use moonshine_save::{
        load::{load_on, LoadWorld},
        save::{save_on, SaveWorld},
    };

    const PATH: &str = "/tmp/save_marker_roundtrip_test.ron";

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()))
            .register_type::<Save>()
            .add_observer(save_on::<SaveWorld<With<Save>>>)
            .add_observer(load_on::<LoadWorld<With<Save>>>);
        app
    }

    #[test]
    fn save_marker_roundtrips_through_save_file() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .run_system_once(|mut commands: Commands| {
                let entity = commands.spawn(Save).id();
                commands.trigger_save(SaveWorld::<With<Save>>::into_file(PATH));
                entity
            })
            .unwrap();

        let data = std::fs::read_to_string(PATH).unwrap();
        assert!(
            data.contains("explore_game::scene::save::Save"),
            "Save marker missing from save file:\n{data}"
        );

        app.world_mut().despawn(entity);
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.trigger_load(LoadWorld::<With<Save>>::from_file(PATH));
            })
            .unwrap();
        app.update();

        let marked: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Save>>()
            .iter(app.world())
            .collect();
        assert_eq!(marked.len(), 1);
        std::fs::remove_file(PATH).unwrap();
    }
}
