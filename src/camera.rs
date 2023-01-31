use crate::input::{Action, ActionState};
use crate::State;
use bevy::{prelude::*, window::CursorGrabMode};

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(State::Running)
                .with_system(camera_control.before(camera_movement))
                .with_system(cursor_grab)
                .with_system(camera_movement),
        );
    }
}

#[derive(Component, Debug)]
pub struct CameraBounds {
    pub position: Vec3,
    pub extent: Vec3,
    pub gap: f32,
}

impl Default for CameraBounds {
    fn default() -> Self {
        Self {
            position: Vec3::new(-10.0, 5.0, -10.0),
            extent: Vec3::new(20.0, 25.0, 20.0),
            gap: 1.0,
        }
    }
}

#[derive(Component, Debug)]
pub struct CameraControl {
    pub velocity: Vec3,
    pub acceleration: f32,
    pub mouse_sensitivity: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        CameraControl {
            velocity: Vec3::ZERO,
            acceleration: 80.0,
            mouse_sensitivity: 0.02,
        }
    }
}

fn camera_control(
    time: Res<Time>,
    mut camera_query: Query<(&Transform, &CameraBounds, &mut CameraControl)>,
    action_state_query: Query<&ActionState<Action>>,
) {
    let (transform, bounds, mut control) = camera_query.single_mut();
    let action_state = action_state_query.single();
    let acceleration = control.acceleration;
    let mut delta = Vec3::ZERO;

    if action_state.pressed(Action::PanCameraRight)
        && transform.translation.x <= bounds.position.x + bounds.extent.x - bounds.gap
    {
        delta += Vec3::X;
    }

    if action_state.pressed(Action::PanCameraLeft)
        && transform.translation.x >= bounds.position.x + bounds.gap
    {
        delta -= Vec3::X;
    }

    if action_state.pressed(Action::PanCameraUp)
        && transform.translation.z >= bounds.position.z + bounds.gap
    {
        delta -= Vec3::Z;
    }

    if action_state.pressed(Action::PanCameraDown)
        && transform.translation.z <= bounds.position.z + bounds.extent.z - bounds.gap
    {
        delta += Vec3::Z;
    }

    let zoom = action_state.value(Action::ZoomCamera);

    if (zoom > 0.0 && transform.translation.y <= bounds.position.y + bounds.extent.y - bounds.gap)
        || (zoom < 0.0 && transform.translation.y >= bounds.position.y + bounds.gap)
    {
        delta += Vec3::Y * zoom;
    }

    if action_state.pressed(Action::PanCamera) {
        let camera_pan = action_state.axis_pair(Action::PanCameraMotion).unwrap();
        delta += Vec3::new(camera_pan.y(), 0.0, -camera_pan.x()) * control.mouse_sensitivity;
    }

    if transform.translation.x < bounds.position.x {
        delta += Vec3::X;
    }

    if transform.translation.x > bounds.position.x + bounds.extent.x {
        delta -= Vec3::X;
    }

    if transform.translation.z < bounds.position.z {
        delta += Vec3::Z;
    }

    if transform.translation.z > bounds.position.z + bounds.extent.z {
        delta -= Vec3::Z;
    }

    if transform.translation.y < bounds.position.y {
        delta += Vec3::Y;
    }

    if transform.translation.y > bounds.position.y + bounds.extent.y {
        delta -= Vec3::Y;
    }

    control.velocity += delta.normalize_or_zero() * time.delta_seconds() * acceleration;
}

fn camera_movement(time: Res<Time>, mut camera_query: Query<(&mut Transform, &mut CameraControl)>) {
    let (mut transform, mut control) = camera_query.single_mut();
    transform.translation += control.velocity * time.delta_seconds();
    control.velocity *= 1.0 - 4.0 * time.delta_seconds();
}

fn cursor_grab(mut windows: ResMut<Windows>, action_state_query: Query<&ActionState<Action>>) {
    let action_state = action_state_query.single();
    if let Some(window) = windows.get_primary_mut() {
        if action_state.just_pressed(Action::PanCamera) {
            window.set_cursor_grab_mode(CursorGrabMode::Locked);
            window.set_cursor_visibility(false);
        }

        if action_state.just_released(Action::PanCamera) {
            window.set_cursor_grab_mode(CursorGrabMode::None);
            window.set_cursor_visibility(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::camera::{camera_control, camera_movement, CameraBounds, CameraControl};
    use crate::input::Action;
    use bevy::{prelude::*, time::Time, utils::Duration};
    use leafwing_input_manager::prelude::ActionState;

    fn init_bare_app() -> App {
        let mut app = App::new();
        app.add_system(camera_control.before(camera_movement));
        app.add_system(camera_movement);

        let mut time = Time::default();
        time.update();
        app.insert_resource(time);

        let mut time = app.world.resource_mut::<Time>();
        let last_update = time.last_update().unwrap();
        time.update_with_instant(last_update + Duration::from_millis(10));

        app.world.spawn(ActionState::<Action>::default());

        app
    }

    #[test]
    fn becomes_stationary() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn((
                Transform::from_xyz(0.0, 10.0, 0.0),
                CameraControl {
                    velocity: Vec3::X,
                    ..default()
                },
                CameraBounds::default(),
            ))
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
        assert!(cameracontrol.velocity.abs_diff_eq(Vec3::ZERO, 0.01));
    }

    #[test]
    fn press_changes_velocity() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn((
                Transform::from_xyz(0.0, 10.0, 0.0),
                CameraControl::default(),
                CameraBounds::default(),
            ))
            .id();

        let mut action_state = app
            .world
            .query::<&mut ActionState<Action>>()
            .single_mut(&mut app.world);
        action_state.press(Action::PanCameraRight);

        app.update();

        let cameracontrol = app.world.get::<CameraControl>(camera_id).unwrap();
        assert!(cameracontrol
            .velocity
            .abs_diff_eq(Vec3::new(0.76, 0.0, 0.0), 0.01));
    }

    #[test]
    fn press_inside_gap_does_nothing() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn((
                Transform::from_xyz(9.8, 10.0, 0.0),
                CameraControl::default(),
                CameraBounds::default(),
            ))
            .id();

        let mut action_state = app
            .world
            .query::<&mut ActionState<Action>>()
            .single_mut(&mut app.world);
        action_state.press(Action::PanCameraRight);

        app.update();

        let cameracontrol = app.world.get::<CameraControl>(camera_id).unwrap();
        assert_eq!(cameracontrol.velocity, Vec3::ZERO);
    }

    #[test]
    fn outside_bounds_gets_pushed_back() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn((
                Transform::from_xyz(-10.2, 10.0, 0.0),
                CameraControl::default(),
                CameraBounds::default(),
            ))
            .id();

        app.update();

        let cameracontrol = app.world.get::<CameraControl>(camera_id).unwrap();
        assert!(cameracontrol
            .velocity
            .abs_diff_eq(Vec3::new(0.76, 0.0, 0.0), 0.01));
    }
}
