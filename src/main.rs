use bevy::{
    asset::AssetServerSettings, prelude::*, render::texture::ImageSettings, window::PresentMode,
};
use bevy_mod_picking::{PickableBundle, PickingCameraBundle};
use rand::Rng;

mod action;
mod camera;
mod fog;
mod hex;
mod indicator;
mod input;
mod interface;
mod map;
mod party;
mod zone;
mod zone_material;

use action::{handle_move_to, GameAction};
use camera::{CameraBounds, CameraControl, CameraControlPlugin};
use fog::Fog;
use hex::{coord_to_vec3, Hexagon};
use indicator::{update_indicator, Indicator};
use input::InputPlugin;
use interface::InterfacePlugin;
use map::{
    events::Entered, HexCoord, Map, MapComponent, MapLayout, MapPlugin, MapPresence, PathGuided,
};
use party::Party;
use zone::{Terrain, Zone};
use zone_material::{ZoneMaterial, ZoneMaterialPlugin};

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const VIEW_RADIUS: usize = 2;

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
        .add_event::<GameAction>()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(CameraControlPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(InterfacePlugin)
        .add_plugin(MapPlugin)
        .add_plugin(ZoneMaterialPlugin)
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_camera)
        .add_system(log_moves)
        .add_system(handle_move_to)
        .add_system(update_indicator);

    app.run();
}

fn log_moves(mut entered_event: EventReader<Entered>) {
    for event in entered_event.iter() {
        info!("{:?} moved to {:?}", event.entity, event.coordinate);
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
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
) {
    let cloud_texture = asset_server.load("textures/cloud.png");
    let grass_texture = asset_server.load("textures/grass.png");
    let lava_texture = asset_server.load("textures/lava.png");
    let offset = Vec3::new(0.0, 1.0, 0.0);
    let mut rng = rand::thread_rng();
    let maplayout = MapLayout {
        width: 20,
        height: 16,
    };
    let mut map = Map::new(maplayout);
    let cubecoord = HexCoord::new(2, 6);
    for position in maplayout.iter() {
        let terrain = rng.gen();
        let entity = commands
            .spawn_bundle(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(Hexagon { radius: 1.0 })),
                material: match terrain {
                    Terrain::Grass => zone_materials.add(ZoneMaterial {
                        cloud_texture: Some(cloud_texture.clone()),
                        terrain_texture: Some(grass_texture.clone()),
                        visible: 1,
                        explored: 1,
                        time: 0.0,
                    }),
                    Terrain::Lava => zone_materials.add(ZoneMaterial {
                        cloud_texture: Some(cloud_texture.clone()),
                        terrain_texture: Some(lava_texture.clone()),
                        visible: 1,
                        explored: 1,
                        time: 0.0,
                    }),
                },
                transform: Transform::from_translation(coord_to_vec3(position, 1.0)),
                ..default()
            })
            .insert(Zone { position, terrain })
            .insert(Fog {
                visible: false,
                explored: false,
            })
            .insert(bevy_mod_picking::PickableMesh::default())
            .insert(bevy_mod_picking::Hover::default())
            .insert(bevy_mod_picking::NoDeselect)
            .insert(Interaction::default())
            .id();
        map.set(position, Some(entity));
    }
    let map = commands
        .spawn()
        .insert(MapComponent { map, radius: 1.0 })
        .id();
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset_server.load("models/indicator.stl"),
            material: standard_materials.add(Color::rgb(0.165, 0.631, 0.596).into()),
            transform: Transform::from_translation(coord_to_vec3(cubecoord, 1.0) + offset),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(Indicator)
        .insert(Party {
            name: String::from("Alpha Group"),
        })
        .insert(MapPresence {
            map,
            position: cubecoord,
            offset,
            view_radius: VIEW_RADIUS,
        })
        .insert(PathGuided::default());

    let cubecoord = HexCoord::new(4, 5);
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset_server.load("models/indicator.stl"),
            material: standard_materials.add(Color::rgb(0.596, 0.165, 0.0631).into()),
            transform: Transform::from_translation(coord_to_vec3(cubecoord, 1.0) + offset),
            ..default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(Indicator)
        .insert(Party {
            name: String::from("Beta Group"),
        })
        .insert(MapPresence {
            map,
            position: cubecoord,
            offset,
            view_radius: VIEW_RADIUS,
        })
        .insert(PathGuided::default());

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}
