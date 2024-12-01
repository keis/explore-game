mod component;
mod event;
mod plugin;
mod queue;
mod system;

pub use component::{ActionPoints, CampActionAssignment};
pub use plugin::{ActionPlugin, ActionUpdate};
pub use queue::{GameAction, GameActionQueue, GameActionType};

#[cfg(test)]
mod tests {
    use super::{ActionPlugin, ActionPoints};
    use crate::actor::{GroupCommandsExt, Members};
    use bevy::prelude::*;
    use rstest::*;

    #[fixture]
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(ActionPlugin);
        app
    }

    #[rstest]
    fn group_action_points(mut app: App) {
        let group_entity = app
            .world_mut()
            .spawn((Members::default(), ActionPoints::default()))
            .id();
        let speedy_member = app
            .world_mut()
            .spawn(ActionPoints {
                current: 3,
                reset: 3,
            })
            .id();
        app.world_mut()
            .commands()
            .entity(group_entity)
            .add_members(&[speedy_member]);
        app.world_mut().flush();

        let (group_action_points, _members) = app
            .world_mut()
            .query::<(&ActionPoints, &Members)>()
            .single(app.world());
        assert_eq!(group_action_points.current, 3);

        let slow_member = app
            .world_mut()
            .spawn(ActionPoints {
                current: 2,
                reset: 2,
            })
            .id();
        app.world_mut()
            .commands()
            .entity(group_entity)
            .add_members(&[slow_member]);
        app.world_mut().flush();

        let (group_action_points, _members) = app
            .world_mut()
            .query::<(&ActionPoints, &Members)>()
            .single(app.world());
        assert_eq!(group_action_points.current, 2);
    }

    #[rstest]
    fn group_change_without_action_points(mut app: App) {
        let group_entity = app.world_mut().spawn(Members::default()).id();
        let member = app
            .world_mut()
            .spawn(ActionPoints {
                current: 2,
                reset: 2,
            })
            .id();
        app.world_mut()
            .commands()
            .entity(group_entity)
            .add_members(&[member]);
        app.world_mut().flush();
    }
}
