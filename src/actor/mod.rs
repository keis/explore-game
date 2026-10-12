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
    use super::{Character, Enemy, GroupCommandsExt, MemberOf, Members, Party, PartyBundle, Slide};
    use crate::action::ActionPoints;
    use crate::input::Selection;
    use crate::inventory::Inventory;
    use crate::path::PathGuided;
    use bevy::prelude::*;
    use expl_map::{FogRevealer, HexCoord, MapPresence, ViewRadius};
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
            .single(app.world())
            .unwrap();
        assert_eq!(group.len(), 1);
        let member_from_group_entity = group[0];

        let (member_entity, member) = app
            .world_mut()
            .query::<(Entity, &MemberOf)>()
            .single(app.world())
            .unwrap();

        assert_eq!(member_from_group_entity, member_entity);
        assert_eq!(member.get(), group_entity);
    }

    #[rstest]
    fn change_group(mut app: App) {
        let (member_entity, _) = app
            .world_mut()
            .query::<(Entity, &MemberOf)>()
            .single(app.world())
            .unwrap();

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

        let member = app
            .world_mut()
            .query::<&MemberOf>()
            .single(app.world())
            .unwrap();
        assert_eq!(member.0, new_group_entity);
    }

    #[rstest]
    fn remove_from_non_party(mut app: App) {
        let member_entity = app.world_mut().spawn_empty().id();
        let group_entity = app
            .world_mut()
            .commands()
            .spawn(Members::default())
            .add_members(&[member_entity])
            .id();
        app.world_mut().flush();

        app.world_mut()
            .commands()
            .entity(group_entity)
            .remove_members(&[member_entity]);
        app.world_mut().flush();

        assert!(!app.world().entity(group_entity).contains::<Members>());
    }

    #[rstest]
    fn markers_require_bundle_companions(mut app: App) {
        let party = app.world_mut().spawn(Party::default()).id();
        let entity = app.world().entity(party);
        assert!(entity.contains::<Inventory>());
        assert!(entity.contains::<ActionPoints>());
        assert!(entity.contains::<ViewRadius>());
        assert!(entity.contains::<FogRevealer>());
        assert!(entity.contains::<Selection>());
        assert!(entity.contains::<PathGuided>());
        assert!(entity.contains::<Transform>());
        assert!(entity.contains::<Visibility>());

        let character = app.world_mut().spawn(Character::default()).id();
        assert!(app.world().entity(character).contains::<Selection>());

        let enemy = app.world_mut().spawn(Enemy).id();
        let entity = app.world().entity(enemy);
        assert!(entity.contains::<Slide>());
        assert!(entity.contains::<Transform>());
        assert!(entity.contains::<Visibility>());
    }

    #[rstest]
    fn thinned_bundles_spawn_complete_entities(mut app: App) {
        let party = app
            .world_mut()
            .spawn(PartyBundle::new(HexCoord::ZERO, String::from("Test"), 3))
            .id();
        let entity = app.world().entity(party);
        // Explicit bundle values win over require defaults.
        assert_eq!(
            entity
                .get::<Inventory>()
                .unwrap()
                .count_item(Inventory::SUPPLY),
            3
        );
        assert_eq!(
            entity.get::<MapPresence>().unwrap().position,
            HexCoord::ZERO
        );
        // Require defaults fill the thinned fields.
        assert!(entity.contains::<ActionPoints>());
        assert!(entity.contains::<ViewRadius>());
        assert!(entity.contains::<Slide>());
        assert!(entity.contains::<FogRevealer>());
        assert!(entity.contains::<Selection>());
        assert!(entity.contains::<PathGuided>());
        assert!(entity.contains::<Transform>());
        assert!(entity.contains::<Visibility>());
    }
}
