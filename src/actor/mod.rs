mod asset;
mod bundle;
mod command;
mod component;
mod event;
mod plugin;
mod system;
mod system_param;

pub use asset::*;
pub use bundle::*;
pub use command::GroupCommandsExt;
pub use component::*;
pub use event::*;
pub use plugin::ActorPlugin;
pub use system_param::*;

#[cfg(test)]
mod tests {
    use super::{Group, GroupCommandsExt, Members, Party};
    use crate::action::ActionPoints;
    use bevy::prelude::*;
    use rstest::*;

    #[fixture]
    fn app() -> App {
        let mut app = App::new();
        let party_entity = app
            .world_mut()
            .spawn((
                Party::default(),
                Members::default(),
                ActionPoints::default(),
            ))
            .id();
        let member_entity = app
            .world_mut()
            .spawn(ActionPoints {
                current: 2,
                reset: 2,
            })
            .id();
        app.world_mut()
            .commands()
            .entity(party_entity)
            .add_members(&[member_entity]);
        app.world_mut().flush();
        app
    }

    #[rstest]
    fn join_group(mut app: App) {
        let (group_entity, group) = app
            .world_mut()
            .query::<(Entity, &Members)>()
            .single(app.world());
        assert_eq!(group.len(), 1);
        let member_from_group_entity = group[0];

        let (member_entity, member) = app
            .world_mut()
            .query::<(Entity, &Group)>()
            .single(app.world());

        assert_eq!(member_from_group_entity, member_entity);
        assert_eq!(member.get(), group_entity);
    }

    #[rstest]
    fn change_group(mut app: App) {
        let (member_entity, _) = app
            .world_mut()
            .query::<(Entity, &Group)>()
            .single(app.world());

        let new_group_entity = app.world_mut().spawn(Members::default()).id();
        app.world_mut()
            .commands()
            .entity(new_group_entity)
            .add_members(&[member_entity]);
        app.world_mut().flush();

        let group = app
            .world_mut()
            .query::<&Members>()
            .get(app.world(), new_group_entity)
            .unwrap();

        assert_eq!(group.len(), 1);
        assert_eq!(group[0], member_entity);

        let member = app.world_mut().query::<&Group>().single(app.world());
        assert_eq!(member.0, new_group_entity);
    }
}
