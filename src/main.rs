use bevy::render::texture::ImageSettings;
use bevy::{prelude::*, window::PresentMode};
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle, PickingCameraBundle};
use rand::Rng;

mod camera;
mod hex;

use camera::{CameraControl, CameraControlPlugin};
use hex::{HexCoord, Hexagon};

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
        .add_startup_system(spawn_camera)
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(CameraControlPlugin)
        .run();
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
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.165, 0.631, 0.596).into()),
        ..default()
    });
    let offset = Vec3::new(0.0, 0.5, 0.0);
    let mut rng = rand::thread_rng();
    for q in 1..10 {
        for r in 1..8 {
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
                    transform: Transform::from_translation(
                        HexCoord::new(q, r).as_vec3(1.0) + offset,
                    ),
                    ..default()
                })
                .insert_bundle(PickableBundle::default());
        }
    }
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
