use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

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
    mouse_sensitivity: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        CameraControl {
            velocity: Vec3::ZERO,
            acceleration: 4.0,
            mouse_sensitivity: 0.002,
        }
    }
}

fn camera_control(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
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

    let mut scroll: f32 = 0.0;
    for event in mouse_wheel_events.iter() {
        scroll += event.y;
    }

    if scroll.abs() > 0.0 {
        control.velocity += Vec3::Y * scroll * time.delta_seconds() * acceleration;
    }

    transform.translation += control.velocity;
    control.velocity *= 1.0 - 4.0 * time.delta_seconds();

    let mut rotation_move = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        if mouse_input.pressed(MouseButton::Right) {
            rotation_move += event.delta;
        }
    }

    if rotation_move.length_squared() > 0.0 {
        let yaw = Quat::from_rotation_y(-rotation_move.x * control.mouse_sensitivity);
        let pitch = Quat::from_rotation_x(-rotation_move.y * control.mouse_sensitivity);
        transform.rotation = yaw * transform.rotation;
        transform.rotation = transform.rotation * pitch;
    }
}
