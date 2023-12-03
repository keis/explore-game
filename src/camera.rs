use crate::{
    input::{Action, ActionState},
    map::HexCoord,
};
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                camera_control.before(camera_movement),
                camera_target.before(camera_movement),
                cursor_grab,
                camera_movement,
            ),
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

#[derive(Component, Debug)]
pub struct CameraTarget {
    translation: Vec3,
}

impl CameraTarget {
    const OFFSET: Vec3 = Vec3::new(2.0, 10.0, 10.0);

    pub fn from_hexcoord(coord: HexCoord) -> Self {
        Self {
            translation: Vec3::from(coord) + Self::OFFSET,
        }
    }
}

fn camera_control(
    time: Res<Time>,
    mut camera_query: Query<(&Transform, &CameraBounds, &mut CameraControl)>,
    action_state: Res<ActionState<Action>>,
) {
    let (transform, bounds, mut control) = camera_query.single_mut();
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
        delta += Vec3::new(-camera_pan.x(), 0.0, -camera_pan.y()) * control.mouse_sensitivity;
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

fn camera_target(
    mut commands: Commands,
    time: Res<Time>,
    mut camera_query: Query<(Entity, &Transform, &mut CameraControl, &CameraTarget)>,
) {
    if let Ok((entity, transform, mut control, target)) = camera_query.get_single_mut() {
        let acceleration = control.acceleration;
        let delta = target.translation - transform.translation;
        control.velocity += delta.normalize_or_zero() * time.delta_seconds() * acceleration;
        if delta.length_squared() < 1.0 {
            commands.entity(entity).remove::<CameraTarget>();
        }
    }
}

fn camera_movement(time: Res<Time>, mut camera_query: Query<(&mut Transform, &mut CameraControl)>) {
    let (mut transform, mut control) = camera_query.single_mut();
    transform.translation += control.velocity * time.delta_seconds();
    control.velocity *= 1.0 - 4.0 * time.delta_seconds();
}

fn cursor_grab(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    action_state: Res<ActionState<Action>>,
) {
    let Ok(mut window) = window_query.get_single_mut() else {
        return;
    };

    if action_state.just_pressed(Action::PanCamera) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    if action_state.just_released(Action::PanCamera) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

#[cfg(test)]
mod tests {
    use crate::camera::{
        camera_control, camera_movement, camera_target, CameraBounds, CameraControl, CameraTarget,
    };
    use crate::input::Action;
    use bevy::{prelude::*, time::Time, utils::Duration};
    use leafwing_input_manager::prelude::ActionState;

    fn init_bare_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                camera_control.before(camera_movement),
                camera_target.before(camera_movement),
                camera_movement,
            ),
        );

        app.init_resource::<Time>();

        let mut time = app.world.resource_mut::<Time>();
        time.advance_by(Duration::from_millis(10));

        app.insert_resource(ActionState::<Action>::default());

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
            time.advance_by(Duration::from_millis(100));
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

        app.world
            .resource_mut::<ActionState<Action>>()
            .press(Action::PanCameraRight);

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

        app.world
            .resource_mut::<ActionState<Action>>()
            .press(Action::PanCameraRight);

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

    #[test]
    fn moves_to_target() {
        let mut app = init_bare_app();

        let camera_id = app
            .world
            .spawn((
                Transform::from_xyz(0.0, 10.0, 0.0),
                CameraControl::default(),
                CameraBounds::default(),
                CameraTarget {
                    translation: Vec3::new(10.0, 10.0, 10.0),
                },
            ))
            .id();

        for _ in 0..100 {
            let mut time = app.world.resource_mut::<Time>();
            time.advance_by(Duration::from_millis(100));
            app.update()
        }

        let transform = app.world.get::<Transform>(camera_id).unwrap();
        println!("transform {:?}", transform);
        assert!(transform
            .translation
            .abs_diff_eq(Vec3::new(10.0, 10.0, 10.0), 2.0)); // Eh, good enough
    }
}
