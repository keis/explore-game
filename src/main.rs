use bevy::{log::LogPlugin, prelude::*, window::PresentMode};
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::PickingCameraBundle;
use explore_game::{
    action::ActionPlugin,
    assets::MainAssets,
    camera::{CameraBounds, CameraControl, CameraControlPlugin},
    character::Character,
    hex::{coord_to_vec3, Hexagon},
    hexgrid::spiral,
    indicator::update_indicator,
    input::InputPlugin,
    interface::InterfacePlugin,
    map::{
        spawn_game_map_from_prototype, start_map_generation, AddMapPresence, GameMap,
        GenerateMapTask, HexCoord, MapEvent, MapPlugin, MapPosition, MapPresence, Offset, Terrain,
        ViewRadius, Zone, ZoneBundle,
    },
    material::{ZoneMaterial, ZoneMaterialPlugin},
    party::{reset_movement_points, JoinParty, Party, PartyBundle, PartyMember},
    slide::{slide, SlideEvent},
    turn::Turn,
    State, VIEW_RADIUS,
};
use futures_lite::future;
use smallvec::SmallVec;

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;

fn main() {
    let height = 900.0;

    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(Turn { number: 0 })
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
            info!("{:?} moved to {:?}", entity, position);
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
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 20.0, 40.0)
                .looking_at(Vec3::new(8.0, 0.0, 20.0), Vec3::Y),
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

fn zone_material(assets: &Res<MainAssets>, terrain: Terrain) -> ZoneMaterial {
    match terrain {
        Terrain::Ocean => ZoneMaterial {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture: Some(assets.ocean_texture.clone()),
            visible: 1,
            explored: 1,
        },
        Terrain::Mountain => ZoneMaterial {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture: Some(assets.mountain_texture.clone()),
            visible: 1,
            explored: 1,
        },
        Terrain::Forest => ZoneMaterial {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture: Some(assets.forest_texture.clone()),
            visible: 1,
            explored: 1,
        },
    }
}

fn spawn_zone(
    commands: &mut Commands,
    assets: &Res<MainAssets>,
    hexmesh: &Handle<Mesh>,
    zone_materials: &mut ResMut<Assets<ZoneMaterial>>,
    position: HexCoord,
    terrain: Terrain,
) -> Entity {
    commands
        .spawn((
            ZoneBundle {
                position: MapPosition(position),
                zone: Zone { terrain },
                ..default()
            },
            MaterialMeshBundle {
                mesh: hexmesh.clone(),
                material: zone_materials.add(zone_material(assets, terrain)),
                transform: Transform::from_translation(coord_to_vec3(position))
                    .with_rotation(Quat::from_rotation_y((90f32).to_radians())),
                ..default()
            },
        ))
        .id()
}

fn spawn_party(
    commands: &mut Commands,
    assets: &Res<MainAssets>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
    position: HexCoord,
    color: Color,
    name: String,
) -> Entity {
    let offset = Vec3::new(0.0, 1.0, 0.0);
    commands
        .spawn((
            PbrBundle {
                mesh: assets.indicator_mesh.clone(),
                material: standard_materials.add(color.into()),
                transform: Transform::from_translation(coord_to_vec3(position) + offset),
                ..default()
            },
            PartyBundle {
                party: Party {
                    name,
                    movement_points: 2,
                    supplies: 1,
                    members: SmallVec::new(),
                },
                offset: Offset(offset),
                view_radius: ViewRadius(VIEW_RADIUS),
                ..default()
            },
        ))
        .id()
}

fn spawn_scene(
    mut commands: Commands,
    assets: Res<MainAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
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

    let hexmesh = meshes.add(Mesh::from(Hexagon { radius: 1.0 }));
    let map =
        spawn_game_map_from_prototype(&mut commands, &prototype, |commands, position, terrain| {
            spawn_zone(
                commands,
                &assets,
                &hexmesh,
                &mut zone_materials,
                position,
                terrain,
            )
        });

    let groupcoord = spiral((2, 6).into())
        .find(|&c| prototype.get(c).map_or(false, |&t| t != Terrain::Ocean))
        .unwrap();
    let alpha_group = spawn_party(
        &mut commands,
        &assets,
        &mut standard_materials,
        groupcoord,
        Color::rgb(0.165, 0.631, 0.596),
        String::from("Alpha Group"),
    );
    commands.add(AddMapPresence {
        map,
        presence: alpha_group,
        position: groupcoord,
    });
    let character1 = commands
        .spawn(Character {
            name: String::from("Alice"),
        })
        .insert(PartyMember { party: alpha_group })
        .id();
    let character2 = commands
        .spawn(Character {
            name: String::from("Bob"),
        })
        .id();
    commands.add(JoinParty {
        party: alpha_group,
        members: SmallVec::from_slice(&[character1, character2]),
    });

    let groupcoord = spiral((4, 5).into())
        .find(|&c| prototype.get(c).map_or(false, |&t| t != Terrain::Ocean))
        .unwrap();
    let beta_group = spawn_party(
        &mut commands,
        &assets,
        &mut standard_materials,
        groupcoord,
        Color::rgb(0.596, 0.165, 0.631),
        String::from("Beta Group"),
    );
    commands.add(AddMapPresence {
        map,
        presence: beta_group,
        position: groupcoord,
    });
    let character3 = commands
        .spawn(Character {
            name: String::from("Carol"),
        })
        .id();
    commands.add(JoinParty {
        party: beta_group,
        members: SmallVec::from_slice(&[character3]),
    });
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
