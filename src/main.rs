use bevy::{prelude::*, render::texture::ImageSettings, window::PresentMode};
use bevy_mod_picking::{
    DefaultPickingPlugins, HoverEvent, PickableBundle, PickingCameraBundle, PickingEvent,
};
use rand::Rng;
use std::collections::VecDeque;

mod camera;
mod fog;
mod hex;
mod map;
mod zone;
mod zone_material;

use camera::{CameraBounds, CameraControl, CameraControlPlugin};
use fog::Fog;
use hex::{HexCoord, Hexagon};
use map::{find_path, Map, MapComponent, MapLayout};
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
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_interface)
        .add_startup_system(spawn_camera)
        .add_system(move_map_walker)
        .add_system(log_moves)
        .add_system_to_stage(CoreStage::PostUpdate, handle_picking_events)
        .add_system_to_stage(CoreStage::PostUpdate, update_visibility)
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(CameraControlPlugin)
        .add_plugin(ZoneMaterialPlugin)
        .add_event::<HexEntered>();

    app.run();
}

#[derive(Component)]
pub struct HexPositioned {
    pub position: HexCoord,
    pub radius: f32,
    pub offset: Vec3,
}

#[derive(Component)]
pub struct MapWalker {
    pub progress: f32,
    pub path: VecDeque<HexCoord>,
}

#[derive(Component)]
pub struct ZoneText;

fn log_moves(mut hex_entered_event: EventReader<HexEntered>) {
    for event in hex_entered_event.iter() {
        info!("{:?} moved to {:?}", event.entity, event.coordinate);
    }
}

fn update_visibility(
    mut hex_entered_event: EventReader<HexEntered>,
    positioned_query: Query<&HexPositioned>,
    mut zone_query: Query<(&Zone, &mut Fog)>,
) {
    let mut moved = false;
    for _event in hex_entered_event.iter() {
        moved = true;
    }

    if moved {
        let positioned = positioned_query
            .get_single()
            .expect("has positioned entity");
        for (zone, mut fog) in zone_query.iter_mut() {
            fog.visible = zone.position.distance(&positioned.position) <= VIEW_RADIUS;
            fog.explored = fog.explored || fog.visible;
        }
    }
}

pub fn move_map_walker(
    time: Res<Time>,
    mut positioned_query: Query<(Entity, &mut MapWalker, &mut HexPositioned, &mut Transform)>,
    mut hex_entered_event: EventWriter<HexEntered>,
) {
    let (entity, mut mapwalker, mut positioned, mut transform) = positioned_query.single_mut();

    if mapwalker.path.len() == 0 {
        return;
    }

    mapwalker.progress += time.delta_seconds();
    if mapwalker.progress >= 1.0 {
        positioned.position = mapwalker.path.pop_front().expect("path has element");
        hex_entered_event.send(HexEntered {
            entity,
            coordinate: positioned.position,
        });
        mapwalker.progress = 0.0;
    }

    if let Some(next) = mapwalker.path.front() {
        let orig_translation = positioned.position.as_vec3(positioned.radius) + positioned.offset;
        let new_translation = next.as_vec3(positioned.radius) + positioned.offset;
        transform.translation = orig_translation.lerp(new_translation, mapwalker.progress);
    }
}

pub struct HexEntered {
    entity: Entity,
    coordinate: HexCoord,
}

pub fn handle_picking_events(
    mut events: EventReader<PickingEvent>,
    zone_query: Query<&Zone>,
    map_query: Query<&MapComponent>,
    mut positioned_query: Query<(&mut MapWalker, &HexPositioned)>,
    mut zone_text_query: Query<&mut Text, With<ZoneText>>,
) {
    let map = &map_query.get_single().expect("has exactly one map").map;
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(e) => {
                if let Ok(zone) = zone_query.get(*e) {
                    info!("Clicked a zone: {:?}", zone);
                    let (mut mapwalker, positioned) = positioned_query.single_mut();
                    if let Some((path, _length)) =
                        find_path(positioned.position, zone.position, &|c: &HexCoord| {
                            if let Some(entity) = map.get(*c) {
                                if let Ok(zone) = zone_query.get(entity) {
                                    return zone.terrain != Terrain::Lava;
                                }
                            }
                            false
                        })
                    {
                        mapwalker.path = VecDeque::from(path);
                        mapwalker.path.pop_front();
                    }
                }
            }
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => {
                if let Ok(zone) = zone_query.get(*e) {
                    for mut text in &mut zone_text_query {
                        text.sections[0].value = format!("{:?}", zone.position);
                    }
                }
            }
            _ => {}
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
                transform: Transform::from_translation(position.as_vec3(1.0)),
                ..default()
            })
            .insert(Zone { position, terrain })
            .insert(Fog {
                visible: position.distance(&cubecoord) <= VIEW_RADIUS,
                explored: position.distance(&cubecoord) <= VIEW_RADIUS + 2,
            })
            .insert_bundle(PickableBundle::default())
            .id();
        map.set(position, Some(entity));
    }
    commands.spawn().insert(MapComponent { map });
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: standard_materials.add(Color::rgb(0.165, 0.631, 0.596).into()),
            transform: Transform::from_translation(HexCoord::new(2, 6).as_vec3(1.0) + offset),
            ..default()
        })
        .insert(HexPositioned {
            position: cubecoord,
            radius: 1.0,
            offset,
        })
        .insert(MapWalker {
            progress: 0.0,
            path: VecDeque::new(),
        });

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 8.0, 4.0),
        ..default()
    });
}

fn spawn_interface(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(
            TextBundle::from_section(
                "Zone: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            )
            .with_text_alignment(TextAlignment::TOP_CENTER)
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ZoneText);
}
