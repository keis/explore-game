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
    use super::{command::AddMembers, system::derive_party_movement, Group, Members, Party};
    use crate::creature::Movement;
    use bevy::{ecs::world::Command, prelude::*};
    use rstest::*;
    use smallvec::SmallVec;

    #[fixture]
    fn app() -> App {
        let mut app = App::new();
        app.add_systems(Update, derive_party_movement);
        let party_entity = app
            .world_mut()
            .spawn((Party::default(), Members::default(), Movement::default()))
            .id();
        let member_entity = app
            .world_mut()
            .spawn(Movement {
                current: 2,
                reset: 2,
            })
            .id();
        let addmembers = AddMembers {
            group: party_entity,
            members: SmallVec::from_slice(&[member_entity]),
        };
        addmembers.apply(app.world_mut());
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
        let addmembers = AddMembers {
            group: new_group_entity,
            members: SmallVec::from_slice(&[member_entity]),
        };
        addmembers.apply(app.world_mut());

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

    #[rstest]
    fn party_movement(mut app: App) {
        let (mut movement, _member) = app
            .world_mut()
            .query::<(&mut Movement, &Group)>()
            .single_mut(app.world_mut());
        movement.current = 3;

        app.update();

        let (party_movement, _party) = app
            .world_mut()
            .query::<(&Movement, &Party)>()
            .single(app.world());
        assert_eq!(party_movement.current, 3);
    }
}
