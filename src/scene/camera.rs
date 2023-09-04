use crate::camera::{CameraBounds, CameraControl};
use bevy::prelude::*;
use bevy_mod_picking::prelude::RaycastPickCamera;

pub fn spawn_camera(mut commands: Commands) {
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
        RaycastPickCamera::default(),
    ));
}
