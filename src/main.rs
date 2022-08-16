use bevy::{
    pbr::RenderMaterials,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::*, renderer::RenderQueue, texture::ImageSettings, Extract, RenderApp,
        RenderStage,
    },
    window::PresentMode,
};
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

use camera::{CameraControl, CameraControlPlugin};
use fog::Fog;
use hex::{HexCoord, Hexagon};
use map::{find_path, Map, MapComponent, MapLayout};
use zone::{Terrain, Zone};

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
        .add_system(move_hex_positioned)
        .add_system(log_moves)
        .add_system_to_stage(CoreStage::PostUpdate, handle_picking_events)
        .add_system_to_stage(CoreStage::PostUpdate, update_visibility)
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(CameraControlPlugin)
        .add_plugin(MaterialPlugin::<ZoneMaterial>::default())
        .add_event::<HexEntered>();

    app.sub_app_mut(RenderApp)
        .add_system_to_stage(RenderStage::Extract, extract_zone)
        .add_system_to_stage(RenderStage::Prepare, prepare_zone_material);

    app.run();
}

#[derive(Component)]
pub struct HexPositioned {
    pub position: HexCoord,
    pub radius: f32,
    pub offset: Vec3,
    pub progress: f32,
    pub path: VecDeque<HexCoord>,
}

#[derive(Component)]
pub struct ZoneText;

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "05f50382-7218-4860-8c4c-06dbd66694db"]
pub struct ZoneMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Option<Handle<Image>>,
    #[uniform(2)]
    pub visible: u32,
    #[uniform(2)]
    pub explored: u32,
}

impl Material for ZoneMaterial {
    fn fragment_shader() -> ShaderRef {
        "zone_material.wgsl".into()
    }
}

#[derive(Clone, ShaderType)]
struct ZoneMaterialUniformData {
    visible: u32,
    explored: u32,
}

fn extract_zone(
    mut commands: Commands,
    zone_query: Extract<Query<(Entity, &Fog, &Handle<ZoneMaterial>)>>,
) {
    for (entity, fog, handle) in zone_query.iter() {
        commands
            .get_or_spawn(entity)
            .insert(*fog)
            .insert(handle.clone());
    }
}

fn prepare_zone_material(
    materials: Res<RenderMaterials<ZoneMaterial>>,
    zone_query: Query<(&Fog, &Handle<ZoneMaterial>)>,
    render_queue: Res<RenderQueue>,
) {
    for (fog, handle) in &zone_query {
        if let Some(material) = materials.get(handle) {
            for binding in material.bindings.iter() {
                if let OwnedBindingResource::Buffer(cur_buffer) = binding {
                    let mut buffer = encase::UniformBuffer::new(Vec::new());
                    buffer
                        .write(&ZoneMaterialUniformData {
                            visible: fog.visible as u32,
                            explored: fog.explored as u32,
                        })
                        .unwrap();
                    render_queue.write_buffer(cur_buffer, 0, buffer.as_ref());
                }
            }
        }
    }
}

fn log_moves(mut hex_entered_event: EventReader<HexEntered>) {
    for event in hex_entered_event.iter() {
        info!("moved to {:?}", event.coordinate);
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

pub fn move_hex_positioned(
    time: Res<Time>,
    mut positioned_query: Query<(Entity, &mut HexPositioned, &mut Transform)>,
    mut hex_entered_event: EventWriter<HexEntered>,
) {
    let (entity, mut positioned, mut transform) = positioned_query.single_mut();

    if positioned.path.len() == 0 {
        return;
    }

    positioned.progress += time.delta_seconds();
    if positioned.progress >= 1.0 {
        positioned.position = positioned.path.pop_front().expect("path has element");
        hex_entered_event.send(HexEntered {
            entity,
            coordinate: positioned.position,
        });
        positioned.progress = 0.0;
    }

    if let Some(next) = positioned.path.front() {
        let orig_translation = positioned.position.as_vec3(positioned.radius) + positioned.offset;
        let new_translation = next.as_vec3(positioned.radius) + positioned.offset;
        transform.translation = orig_translation.lerp(new_translation, positioned.progress);
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
    mut positioned_query: Query<&mut HexPositioned>,
    mut zone_text_query: Query<&mut Text, With<ZoneText>>,
) {
    let map = &map_query.get_single().expect("has exactly one map").map;
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(e) => {
                if let Ok(zone) = zone_query.get(*e) {
                    info!("Clicked a zone: {:?}", zone);
                    let mut positioned = positioned_query.single_mut();
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
                        positioned.path = VecDeque::from(path);
                        positioned.path.pop_front();
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
            transform: Transform::from_xyz(-20.0, 20.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(CameraControl::default())
        .insert_bundle(PickingCameraBundle::default());
}

fn spawn_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
) {
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
                        texture: Some(grass_texture.clone()),
                        visible: 1,
                        explored: 1,
                    }),
                    Terrain::Lava => zone_materials.add(ZoneMaterial {
                        texture: Some(lava_texture.clone()),
                        visible: 1,
                        explored: 1,
                    }),
                },
                transform: Transform::from_translation(position.as_vec3(1.0)),
                ..default()
            })
            .insert(Zone { position, terrain })
            .insert(Fog {
                visible: position.distance(&cubecoord) <= VIEW_RADIUS,
                explored: position.distance(&cubecoord) <= VIEW_RADIUS,
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
            progress: 0.0,
            offset,
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
