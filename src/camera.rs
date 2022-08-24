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

#[derive(Debug)]
pub struct CameraBounds {
    pub position: Vec3,
    pub extent: Vec3,
    pub gap: f32,
}

#[derive(Component, Debug)]
pub struct CameraControl {
    pub bounds: CameraBounds,
    pub velocity: Vec3,
    pub acceleration: f32,
    pub mouse_sensitivity: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        CameraControl {
            bounds: CameraBounds {
                position: Vec3::new(-10.0, 5.0, -10.0),
                extent: Vec3::new(20.0, 25.0, 20.0),
                gap: 1.0,
            },
            velocity: Vec3::ZERO,
            acceleration: 80.0,
            mouse_sensitivity: 0.002,
        }
    }
}

pub fn camera_control(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut CameraControl)>,
) {
    let (mut transform, mut control) = camera_query.single_mut();
    let acceleration = control.acceleration;
    let mut delta = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::Up)
        && transform.translation.x
            <= control.bounds.position.x + control.bounds.extent.x - control.bounds.gap
    {
        delta += Vec3::X;
    }

    if keyboard_input.pressed(KeyCode::Down)
        && transform.translation.x >= control.bounds.position.x + control.bounds.gap
    {
        delta -= Vec3::X;
    }

    if keyboard_input.pressed(KeyCode::Left)
        && transform.translation.z >= control.bounds.position.z + control.bounds.gap
    {
        delta -= Vec3::Z;
    }

    if keyboard_input.pressed(KeyCode::Right)
        && transform.translation.z
            <= control.bounds.position.z + control.bounds.extent.z - control.bounds.gap
    {
        delta += Vec3::Z;
    }

    let mut scroll: f32 = 0.0;
    for event in mouse_wheel_events.iter() {
        scroll += event.y;
    }

    if (scroll > 0.0
        && transform.translation.y
            <= control.bounds.position.y + control.bounds.extent.y - control.bounds.gap)
        || (scroll < 0.0
            && transform.translation.y >= control.bounds.position.y + control.bounds.gap)
    {
        delta += Vec3::Y * scroll;
    }

    if transform.translation.x < control.bounds.position.x {
        delta += Vec3::X;
    }

    if transform.translation.x > control.bounds.position.x + control.bounds.extent.x {
        delta -= Vec3::X;
    }

    if transform.translation.z < control.bounds.position.z {
        delta += Vec3::Z;
    }

    if transform.translation.z > control.bounds.position.z + control.bounds.extent.z {
        delta -= Vec3::Z;
    }

    if transform.translation.y < control.bounds.position.y {
        delta += Vec3::Y;
    }

    if transform.translation.y > control.bounds.position.y + control.bounds.extent.y {
        delta -= Vec3::Y;
    }

    control.velocity += delta.normalize_or_zero() * time.delta_seconds() * acceleration;
    transform.translation += control.velocity * time.delta_seconds();
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

#[cfg(test)]
mod tests {
    use crate::{CameraControl, CameraControlPlugin};
    use bevy::{
        input::mouse::{MouseMotion, MouseWheel},
        prelude::*,
        time::Time,
        utils::Duration,
    };

    fn init_bare_app() -> App {
        let mut app = App::new();
        app.add_plugin(CameraControlPlugin);

        let mut time = Time::default();
        time.update();
        app.insert_resource(time);

        let keyboard_input = Input::<KeyCode>::default();
        app.insert_resource(keyboard_input);

        let mouse_input = Input::<MouseButton>::default();
        app.insert_resource(mouse_input);

        app.add_event::<MouseMotion>();
        app.add_event::<MouseWheel>();

        let mut time = app.world.resource_mut::<Time>();
        let last_update = time.last_update().unwrap();
        time.update_with_instant(last_update + Duration::from_millis(10));

        app
    }

    #[test]
    fn becomes_stationary() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn()
            .insert(Transform::from_xyz(0.0, 10.0, 0.0))
            .insert(CameraControl {
                velocity: Vec3::X,
                ..default()
            })
            .id();

        app.update();

        let cameracontrol = app.world.get::<CameraControl>(camera_id).unwrap();
        assert!(cameracontrol.velocity.length() < 1.0);

        for _ in 0..100 {
            let mut time = app.world.resource_mut::<Time>();
            let last_update = time.last_update().unwrap();
            time.update_with_instant(last_update + Duration::from_millis(100));
            app.update()
        }

        let cameracontrol = app.world.get::<CameraControl>(camera_id).unwrap();
        assert!(cameracontrol.velocity.length() < 1e-10);
    }

    #[test]
    fn press_changes_velocity() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn()
            .insert(Transform::from_xyz(0.0, 10.0, 0.0))
            .insert(CameraControl::default())
            .id();

        let mut keyboard_input = app.world.resource_mut::<Input<KeyCode>>();
        keyboard_input.press(KeyCode::Up);

        app.update();

        let cameracontrol = app.world.get::<CameraControl>(camera_id).unwrap();
        assert!(cameracontrol.velocity.length() > 0.0);
        assert!(cameracontrol.velocity.length() < 1.0);
        // FIXME: Figure out why double normalize() is required
        assert_eq!(cameracontrol.velocity.normalize().normalize(), Vec3::X);
    }

    #[test]
    fn press_inside_gap_does_nothing() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn()
            .insert(Transform::from_xyz(9.8, 10.0, 0.0))
            .insert(CameraControl::default())
            .id();

        let mut keyboard_input = app.world.resource_mut::<Input<KeyCode>>();
        keyboard_input.press(KeyCode::Up);

        app.update();

        let cameracontrol = app.world.get::<CameraControl>(camera_id).unwrap();
        assert_eq!(cameracontrol.velocity, Vec3::ZERO);
    }

    #[test]
    fn outside_bounds_gets_pushed_back() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn()
            .insert(Transform::from_xyz(-10.2, 10.0, 0.0))
            .insert(CameraControl::default())
            .id();

        app.update();

        let cameracontrol = app.world.get::<CameraControl>(camera_id).unwrap();
        assert!(cameracontrol.velocity.length() > 0.0);
        assert!(cameracontrol.velocity.length() < 1.0);
        assert_eq!(cameracontrol.velocity.normalize().normalize(), Vec3::X);
    }
}
