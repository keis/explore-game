use bevy::{
    asset::AssetServerSettings, prelude::*, render::texture::ImageSettings, window::PresentMode,
};
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::{PickableBundle, PickingCameraBundle};

mod action;
mod camera;
mod camp;
mod character;
mod fog;
mod hex;
mod indicator;
mod input;
mod interface;
mod map;
mod party;
mod slide;
mod zone;
mod zone_material;

use action::ActionPlugin;
use camera::{CameraBounds, CameraControl, CameraControlPlugin};
use character::Character;
use fog::Fog;
use hex::{coord_to_vec3, Hexagon};
use indicator::{update_indicator, Indicator};
use input::InputPlugin;
use interface::InterfacePlugin;
use map::{
    AddMapPresence, HexCoord, MapComponent, MapEvent, MapLayout, MapPlugin, MapPosition,
    MapPresence, MapPrototype, MapStorage, Offset, PathGuided, ViewRadius,
};
use party::{reset_movement_points, JoinParty, Party, PartyMember};
use slide::{slide, Slide, SlideEvent};
use smallvec::SmallVec;
use zone::{Terrain, Zone};
use zone_material::{ZoneMaterial, ZoneMaterialPlugin};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const VIEW_RADIUS: usize = 2;

#[derive(Debug)]
pub struct Turn {
    number: u32,
}

fn main() {
    let height = 900.0;

    let mut app = App::new();

    app.insert_resource(ClearColor(CLEAR))
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(WindowDescriptor {
            width: height * ASPECT_RATIO,
            height,
            title: "Explore Game".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..default()
        })
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(Turn { number: 0 })
        .add_plugins(DefaultPlugins)
        .init_collection::<MainAssets>()
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(CameraControlPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(InterfacePlugin)
        .add_plugin(MapPlugin)
        .add_plugin(ZoneMaterialPlugin)
        .add_plugin(ActionPlugin)
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_camera)
        .add_system(log_moves)
        .add_system(update_indicator)
        .add_system(reset_movement_points)
        .add_system(slide)
        .add_event::<SlideEvent>();

    app.run();
}

#[derive(AssetCollection)]
pub struct MainAssets {
    #[asset(path = "textures/cloud.png")]
    cloud_texture: Handle<Image>,
    #[asset(path = "textures/grass.png")]
    grass_texture: Handle<Image>,
    #[asset(path = "textures/lava.png")]
    lava_texture: Handle<Image>,
    #[asset(path = "models/indicator.stl")]
    indicator_mesh: Handle<Mesh>,
    #[asset(path = "models/tent.stl")]
    tent_mesh: Handle<Mesh>,
}

fn log_moves(
    mut map_events: EventReader<MapEvent>,
    presence_query: Query<&MapPresence>,
    map_query: Query<&MapComponent>,
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
                    for other in map
                        .storage
                        .presence(presence.position)
                        .filter(|e| *e != entity)
                    {
                        info!("{:?} is here", other);
                    }
                }
            }
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-10.0, 20.0, 20.0)
                .with_rotation(Quat::from_axis_angle(Vec3::new(-0.4, -0.8, -0.4), 1.6)),
            ..default()
        })
        .insert(CameraControl {
            bounds: CameraBounds {
                position: Vec3::new(-10.0, 5.0, 0.0),
                extent: Vec3::new(15.0, 25.0, 40.0),
                gap: 1.0,
            },
            ..default()
        })
        .insert_bundle(PickingCameraBundle::default());
}

fn spawn_scene(
    mut commands: Commands,
    assets: Res<MainAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
) {
    let offset = Vec3::new(0.0, 1.0, 0.0);
    let maplayout = MapLayout {
        width: 20,
        height: 16,
    };
    let mapprototype = MapPrototype::generate(maplayout);
    let mut mapstorage = MapStorage::new(maplayout);
    let cubecoord = HexCoord::new(2, 6);
    for position in maplayout.iter() {
        let zone = mapprototype.get(position);
        let entity = commands
            .spawn_bundle(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(Hexagon { radius: 1.0 })),
                material: match zone.terrain {
                    Terrain::Grass => zone_materials.add(ZoneMaterial {
                        cloud_texture: Some(assets.cloud_texture.clone()),
                        terrain_texture: Some(assets.grass_texture.clone()),
                        visible: 1,
                        explored: 1,
                        time: 0.0,
                    }),
                    Terrain::Lava => zone_materials.add(ZoneMaterial {
                        cloud_texture: Some(assets.cloud_texture.clone()),
                        terrain_texture: Some(assets.lava_texture.clone()),
                        visible: 1,
                        explored: 1,
                        time: 0.0,
                    }),
                },
                transform: Transform::from_translation(coord_to_vec3(position, 1.0)),
                ..default()
            })
            .insert(MapPosition(position))
            .insert(zone)
            .insert(Fog {
                visible: false,
                explored: false,
            })
            .insert(bevy_mod_picking::PickableMesh::default())
            .insert(bevy_mod_picking::Hover::default())
            .insert(bevy_mod_picking::NoDeselect)
            .insert(Interaction::default())
            .id();
        mapstorage.set(position, Some(entity));
    }
    let map = commands
        .spawn()
        .insert(MapComponent {
            storage: mapstorage,
            radius: 1.0,
        })
        .id();
    let alpha_group = commands
        .spawn_bundle(PbrBundle {
            mesh: assets.indicator_mesh.clone(),
            material: standard_materials.add(Color::rgb(0.165, 0.631, 0.596).into()),
            transform: Transform::from_translation(coord_to_vec3(cubecoord, 1.0) + offset),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(Indicator)
        .insert(Party {
            name: String::from("Alpha Group"),
            movement_points: 0,
            supplies: 1,
            members: SmallVec::new(),
        })
        .insert(Offset(offset))
        .insert(ViewRadius(VIEW_RADIUS))
        .insert(PathGuided::default())
        .insert(Slide::default())
        .id();
    commands.add(AddMapPresence {
        map,
        presence: alpha_group,
        position: cubecoord,
    });
    let character1 = commands
        .spawn()
        .insert(Character {
            name: String::from("Alice"),
        })
        .insert(PartyMember { party: alpha_group })
        .id();
    let character2 = commands
        .spawn()
        .insert(Character {
            name: String::from("Bob"),
        })
        .id();
    commands.add(JoinParty {
        party: alpha_group,
        members: SmallVec::from_slice(&[character1, character2]),
    });

    let cubecoord = HexCoord::new(4, 5);
    let beta_group = commands
        .spawn_bundle(PbrBundle {
            mesh: assets.indicator_mesh.clone(),
            material: standard_materials.add(Color::rgb(0.596, 0.165, 0.631).into()),
            transform: Transform::from_translation(coord_to_vec3(cubecoord, 1.0) + offset),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(Indicator)
        .insert(Party {
            name: String::from("Beta Group"),
            movement_points: 0,
            supplies: 1,
            members: SmallVec::new(),
        })
        .insert(Offset(offset))
        .insert(ViewRadius(VIEW_RADIUS))
        .insert(PathGuided::default())
        .insert(Slide::default())
        .id();
    commands.add(AddMapPresence {
        map,
        presence: beta_group,
        position: cubecoord,
    });
    let character3 = commands
        .spawn()
        .insert(Character {
            name: String::from("Carol"),
        })
        .id();
    commands.add(JoinParty {
        party: beta_group,
        members: SmallVec::from_slice(&[character3]),
    });

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
}
