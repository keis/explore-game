use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    window::PresentMode,
};
use explore_game::path::Path;
use splines::{Interpolation, Key, Spline};

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (800.0, 600.0).into(),
                title: "Example".to_string(),
                present_mode: PresentMode::Fifo,
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WireframePlugin::default())
        .add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(standard_materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Path {
            spline: Spline::from_vec(vec![
                Key::new(
                    0.0,
                    Vec3::new(0.0, 0.0, 0.0),
                    Interpolation::Bezier(Vec3::new(-2.0, 0.0, 0.5)),
                ),
                Key::new(1.0, Vec3::new(4.0, 0.0, 2.0), Interpolation::default()),
            ]),
            steps: 40,
            stroke: 0.1,
        })),
        MeshMaterial3d(standard_materials.add(Color::srgb(1.0, 0.8, 0.8))),
        Transform::from_translation(Vec3::new(-2.0, 0.5, 0.0)),
        Wireframe,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Path {
            spline: Spline::from_vec(vec![
                Key::new(
                    0.0,
                    Vec3::new(0.0, 0.0, 0.0),
                    Interpolation::Bezier(Vec3::new(0.0, 0.0, 1.0)),
                ),
                Key::new(1.0, Vec3::new(1.0, 0.0, 1.0), Interpolation::default()),
            ]),
            steps: 8,
            stroke: 0.1,
        })),
        MeshMaterial3d(standard_materials.add(Color::srgb(0.8, 1.0, 0.8))),
        Transform::from_translation(Vec3::new(2.0, 0.5, 0.0)),
        Wireframe,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
