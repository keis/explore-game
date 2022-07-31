use bevy::prelude::*;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_control);
    }
}

#[derive(Component)]
pub struct CameraControl {
    velocity: Vec3,
    acceleration: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        CameraControl {
            velocity: Vec3::ZERO,
            acceleration: 4.0,
        }
    }
}

fn camera_control(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &mut CameraControl)>,
) {
    let (mut transform, mut control) = camera_query.single_mut();
    let acceleration = control.acceleration;

    if keyboard_input.pressed(KeyCode::Up) {
        control.velocity += Vec3::X * time.delta_seconds() * acceleration;
    }

    if keyboard_input.pressed(KeyCode::Down) {
        control.velocity -= Vec3::X * time.delta_seconds() * acceleration;
    }

    if keyboard_input.pressed(KeyCode::Left) {
        control.velocity -= Vec3::Z * time.delta_seconds() * acceleration;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        control.velocity += Vec3::Z * time.delta_seconds() * acceleration;
    }

    transform.translation += control.velocity;
    control.velocity *= 0.8;
}
