use bevy::render::texture::ImageSettings;
use bevy::{prelude::*, window::PresentMode};
use bevy_mod_picking::{
    DefaultPickingPlugins, HoverEvent, PickableBundle, PickingCameraBundle, PickingEvent,
};
use rand::Rng;
use std::collections::VecDeque;

mod camera;
mod hex;
mod zone;

use camera::{CameraControl, CameraControlPlugin};
use hex::{find_path, HexCoord, Hexagon};
use zone::Zone;

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;

fn main() {
    let height = 900.0;

    App::new()
        .insert_resource(ClearColor(CLEAR))
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
        .add_system_to_stage(CoreStage::PostUpdate, handle_events)
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(CameraControlPlugin)
        .run();
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

pub fn move_hex_positioned(
    time: Res<Time>,
    mut positioned_query: Query<(&mut HexPositioned, &mut Transform)>,
) {
    let (mut positioned, mut transform) = positioned_query.single_mut();

    if positioned.path.len() == 0 {
        return;
    }

    positioned.progress += time.delta_seconds();
    if positioned.progress >= 1.0 {
        positioned.position = positioned.path.pop_front().expect("path has element");
        info!("moved to {:?}", positioned.position);
        positioned.progress = 0.0;
    }

    if let Some(next) = positioned.path.front() {
        let orig_translation = positioned.position.as_vec3(positioned.radius) + positioned.offset;
        let new_translation = next.as_vec3(positioned.radius) + positioned.offset;
        transform.translation = orig_translation.lerp(new_translation, positioned.progress);
    }
}

pub fn handle_events(
    mut events: EventReader<PickingEvent>,
    zone_query: Query<&Zone>,
    mut positioned_query: Query<&mut HexPositioned>,
    mut zone_text_query: Query<&mut Text, With<ZoneText>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(e) => {
                if let Ok(zone) = zone_query.get(*e) {
                    info!("Clicked a zone: {:?}", zone);
                    let mut positioned = positioned_query.single_mut();
                    if let Some((path, _length)) = find_path(positioned.position, zone.position) {
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let offset = Vec3::new(0.0, 1.0, 0.0);
    let mut rng = rand::thread_rng();
    for q in -10..10 {
        for r in -8..8 {
            let position = HexCoord::new(q - r / 2, r);
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(Hexagon { radius: 1.0 })),
                    material: materials.add(
                        Color::rgb(
                            0.827 + rng.gen_range(-0.1..0.1),
                            0.212 + rng.gen_range(-0.1..0.1),
                            0.51 + rng.gen_range(-0.1..0.1),
                        )
                        .into(),
                    ),
                    transform: Transform::from_translation(position.as_vec3(1.0)),
                    ..default()
                })
                .insert(Zone { position })
                .insert_bundle(PickableBundle::default());
        }
    }
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.165, 0.631, 0.596).into()),
            transform: Transform::from_translation(HexCoord::new(2, 6).as_vec3(1.0) + offset),
            ..default()
        })
        .insert(HexPositioned {
            position: HexCoord::new(2, 6),
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
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
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
