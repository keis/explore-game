use bevy::prelude::*;
pub use leafwing_input_manager::prelude::*;
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_input_manager)
            .add_plugin(InputManagerPlugin::<Action>::default());
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    PanCamera,
    PanCameraMotion,
    PanCameraUp,
    PanCameraDown,
    PanCameraLeft,
    PanCameraRight,
    ZoomCamera,
}

fn spawn_input_manager(mut commands: Commands) {
    commands.spawn_bundle(InputManagerBundle {
        action_state: ActionState::default(),
        input_map: InputMap::default()
            .insert(KeyCode::Up, Action::PanCameraUp)
            .insert(KeyCode::Down, Action::PanCameraDown)
            .insert(KeyCode::Left, Action::PanCameraLeft)
            .insert(KeyCode::Right, Action::PanCameraRight)
            .insert(SingleAxis::mouse_wheel_y(), Action::ZoomCamera)
            .insert(MouseButton::Right, Action::PanCamera)
            .insert(DualAxis::mouse_motion(), Action::PanCameraMotion)
            .build(),
    });
}
