use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    window::PresentMode,
};
use explore_game::path::{path_mesh, Path};
use splines::{Interpolation, Key, Spline};

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
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
        .add_plugin(WireframePlugin)
        .add_startup_system(setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 5.0,
            subdivisions: 1,
        })),
        material: standard_materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(path_mesh(Path {
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
            material: standard_materials.add(Color::rgb(1.0, 0.8, 0.8).into()),
            transform: Transform::from_translation(Vec3::new(-2.0, 0.5, 0.0)),
            ..default()
        },
        Wireframe,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(path_mesh(Path {
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
            material: standard_materials.add(Color::rgb(0.8, 1.0, 0.8).into()),
            transform: Transform::from_translation(Vec3::new(2.0, 0.5, 0.0)),
            ..default()
        },
        Wireframe,
    ));

    commands.spawn(DirectionalLightBundle {
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

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
