use bevy::{log::LogPlugin, prelude::*, window::PresentMode};
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::PickingCameraBundle;
use clap::Parser;
use expl_hexgrid::{spiral, GridLayout};
use expl_wfc::{Seed, SeedType};
use explore_game::{
    action::ActionPlugin,
    assets::MainAssets,
    camera::{CameraBounds, CameraControl, CameraControlPlugin},
    camp::update_camp_view_radius,
    character::{reset_movement_points, spawn_character},
    combat,
    enemy::{move_enemy, spawn_enemy},
    indicator::update_indicator,
    input::InputPlugin,
    interface::InterfacePlugin,
    map::{
        spawn_game_map_from_prototype, spawn_zone, start_map_generation, GameMap, GenerateMapTask,
        HexAssets, HexCoord, MapCommandsExt, MapEvent, MapPlugin, MapPresence, MapSeed, Terrain,
    },
    material::{TerrainMaterial, TerrainMaterialPlugin, ZoneMaterial, ZoneMaterialPlugin},
    party::{derive_party_movement, despawn_empty_party, GroupCommandsExt, PartyBundle},
    slide::{slide, SlideEvent},
    turn::Turn,
    State,
};
use futures_lite::future;

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    seed: Option<Seed>,
}

fn main() {
    let cli = Cli::parse();
    let height = 900.0;
    let seed = cli
        .seed
        .unwrap_or_else(|| Seed::new(SeedType::Square(30, 24)));

    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(Turn { number: 0 })
        .insert_resource(MapSeed(seed))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: height * ASPECT_RATIO,
                        height,
                        title: "Explore Game".to_string(),
                        present_mode: PresentMode::Fifo,
                        resizable: false,
                        ..default()
                    },
                    ..default()
                })
                // See https://github.com/Leafwing-Studios/leafwing-input-manager/issues/285
                .set(LogPlugin {
                    filter: "wgpu=error,bevy_ecs::event=error".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                }),
        )
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(CameraControlPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(InterfacePlugin)
        .add_plugin(MapPlugin)
        .add_plugin(ZoneMaterialPlugin)
        .add_plugin(TerrainMaterialPlugin)
        .add_plugin(ActionPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_light)
        .add_startup_system(start_map_generation)
        .add_event::<SlideEvent>()
        .add_state(State::AssetLoading)
        .add_loading_state(
            LoadingState::new(State::AssetLoading)
                .continue_to_state(State::Running)
                .with_collection::<MainAssets>(),
        )
        .add_system_set(
            SystemSet::on_update(State::Running)
                .with_system(spawn_scene)
                .with_system(log_moves)
                .with_system(update_indicator)
                .with_system(reset_movement_points)
                .with_system(derive_party_movement)
                .with_system(despawn_empty_party)
                .with_system(move_enemy)
                .with_system(update_camp_view_radius)
                .with_system(combat::initiate_combat)
                .with_system(combat::combat_round)
                .with_system(combat::despawn_no_health.after(combat::combat_round))
                .with_system(combat::finish_combat.after(combat::despawn_no_health))
                .with_system(slide),
        )
        .run();
}

fn log_moves(
    mut map_events: EventReader<MapEvent>,
    presence_query: Query<&MapPresence>,
    map_query: Query<&GameMap>,
) {
    for event in map_events.iter() {
        if let MapEvent::PresenceMoved {
            presence: entity,
            position,
            ..
        } = event
        {
            info!("{:?} moved to {}", entity, position);
            if let Ok(presence) = presence_query.get(*entity) {
                if let Ok(map) = map_query.get(presence.map) {
                    for other in map.presence(presence.position).filter(|e| *e != entity) {
                        info!("{:?} is here", other);
                    }
                }
            }
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(30.0, 10.0, 30.0);
    let lookto = Vec3::new(-2.0, -20.0, -20.0);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation)
                .looking_at(translation + lookto, Vec3::Y),
            ..default()
        },
        CameraBounds {
            position: Vec3::new(0.0, 5.0, 10.0),
            extent: Vec3::new(40.0, 25.0, 40.0),
            gap: 1.0,
        },
        CameraControl::default(),
        PickingCameraBundle::default(),
    ));
}

#[allow(clippy::type_complexity)]
fn spawn_scene(
    mut commands: Commands,
    mut params: ParamSet<(
        // spawn_party params
        ParamSet<(Res<MainAssets>, ResMut<Assets<StandardMaterial>>)>,
        // spawn_zone params
        ParamSet<(
            Res<MainAssets>,
            Res<HexAssets>,
            ResMut<Assets<ZoneMaterial>>,
            ResMut<Assets<TerrainMaterial>>,
        )>,
        // spawn_enemy params
        ParamSet<(Res<MainAssets>, ResMut<Assets<StandardMaterial>>)>,
    )>,
    mut generate_map_task: Query<(Entity, &mut GenerateMapTask)>,
) {
    if generate_map_task.is_empty() {
        return;
    }
    let (task_entity, mut task) = generate_map_task.single_mut();
    let prototype = match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(result)) => {
            commands.entity(task_entity).despawn();
            result
        }
        Some(Err(e)) => {
            error!("something went wrong: {}", e);
            commands.entity(task_entity).despawn();
            return;
        }
        None => return,
    };

    let map = spawn_game_map_from_prototype(
        &mut commands,
        &prototype,
        |commands, position, zoneproto| spawn_zone(commands, &mut params.p1(), position, zoneproto),
    );

    let groupcoord = spiral(prototype.layout.center())
        .find(|&c| {
            prototype
                .get(c)
                .map_or(false, |proto| proto.terrain != Terrain::Ocean)
        })
        .unwrap();
    let character1 = spawn_character(&mut commands, String::from("Alice"));
    let character2 = spawn_character(&mut commands, String::from("Bob"));
    let character3 = spawn_character(&mut commands, String::from("Carol"));
    let alpha_group = commands
        .spawn(PartyBundle::new(
            &mut params.p0(),
            groupcoord,
            String::from("Alpha Group"),
            1,
        ))
        .add_members(&[character1, character2, character3])
        .id();
    commands.entity(map).add_presence(alpha_group, groupcoord);

    let enemycoord = spiral(prototype.layout.center() + HexCoord::new(2, 3))
        .find(|&c| {
            prototype
                .get(c)
                .map_or(false, |proto| proto.terrain != Terrain::Ocean)
        })
        .unwrap();
    let enemy = spawn_enemy(&mut commands, &mut params.p2(), enemycoord);
    commands.entity(map).add_presence(enemy, enemycoord);
}

fn spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_3),
            ..default()
        },
        ..default()
    });
}
