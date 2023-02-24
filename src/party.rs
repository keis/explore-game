use crate::{
    assets::MainAssets,
    character::Movement,
    indicator::Indicator,
    map::{coord_to_vec3, DespawnPresence, HexCoord, MapPresence, Offset, PathGuided, ViewRadius},
    slide::Slide,
    VIEW_RADIUS,
};

use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use smallvec::SmallVec;
use std::collections::HashSet;

#[derive(Component, Debug, Default)]
pub struct Party {
    pub name: String,
    pub supplies: u32,
}

#[derive(Bundle, Default)]
pub struct PartyBundle {
    pub party: Party,
    pub group: Group,
    pub movement: Movement,
    pub pickable_bundle: PickableBundle,
    pub indicator: Indicator,
    pub offset: Offset,
    pub view_radius: ViewRadius,
    pub path_guided: PathGuided,
    pub slide: Slide,
}

#[derive(Component, Default)]
pub struct Group {
    pub members: SmallVec<[Entity; 8]>,
}

#[derive(Component)]
pub struct GroupMember {
    pub group: Entity,
}

pub struct JoinGroup {
    pub group: Entity,
    pub members: SmallVec<[Entity; 8]>,
}

pub struct RemoveMembers {
    pub group: Entity,
    pub members: SmallVec<[Entity; 8]>,
}

impl Command for JoinGroup {
    fn write(mut self, world: &mut World) {
        let mut old = HashSet::new();
        for &member in &self.members {
            if let Some(mut group_member) = world.entity_mut(member).get_mut::<GroupMember>() {
                if group_member.group != self.group {
                    old.insert(group_member.group);
                    group_member.group = self.group;
                }
            } else {
                world
                    .entity_mut(member)
                    .insert(GroupMember { group: self.group });
            }
        }

        for old_group_entity in old {
            if let Some(mut old_group) = world.entity_mut(old_group_entity).get_mut::<Group>() {
                old_group
                    .members
                    .retain(|m| !self.members.iter().any(|o| *m == *o));
            }
        }

        let mut group_entity = world.entity_mut(self.group);
        if let Some(mut group) = group_entity.get_mut::<Group>() {
            group.members.append(&mut self.members);
        }
    }
}

impl Command for RemoveMembers {
    fn write(self, world: &mut World) {
        let mut group_entity = world.entity_mut(self.group);
        if let Some(mut group) = group_entity.get_mut::<Group>() {
            group
                .members
                .retain(|member| !self.members.contains(member));
        }
        for member in self.members {
            world.entity_mut(member).remove::<Parent>();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn despawn_empty_party(
    mut commands: Commands,
    party_query: Query<(Entity, &Group, &MapPresence), (With<Party>, Changed<Group>)>,
) {
    for (entity, group, presence) in &party_query {
        if group.members.is_empty() {
            commands.add(DespawnPresence {
                map: presence.map,
                presence: entity,
            });
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn derive_party_movement(
    mut party_query: Query<(&Group, &mut Movement), (With<Party>, Changed<Group>)>,
    movement_query: Query<&Movement, Without<Party>>,
) {
    for (group, mut party_movement) in party_query.iter_mut() {
        party_movement.points = movement_query
            .iter_many(&group.members)
            .map(|m| m.points)
            .min()
            .unwrap_or(0);
    }
}

pub fn spawn_party(
    commands: &mut Commands,
    params: &mut ParamSet<(Res<MainAssets>, ResMut<Assets<StandardMaterial>>)>,
    position: HexCoord,
    name: String,
    supplies: u32,
) -> Entity {
    let offset = Vec3::new(0.0, 1.0, 0.0);
    commands
        .spawn((
            PbrBundle {
                mesh: params.p0().indicator_mesh.clone(),
                material: params.p1().add(Color::rgb(0.165, 0.631, 0.596).into()),
                transform: Transform::from_translation(coord_to_vec3(position) + offset),
                ..default()
            },
            PartyBundle {
                party: Party { name, supplies },
                group: Group::default(),
                offset: Offset(offset),
                view_radius: ViewRadius(VIEW_RADIUS),
                ..default()
            },
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::{derive_party_movement, Group, GroupMember, JoinGroup, Movement, Party};
    use bevy::{ecs::system::Command, prelude::*};
    use rstest::*;
    use smallvec::SmallVec;

    #[fixture]
    fn app() -> App {
        let mut app = App::new();
        app.add_system(derive_party_movement);
        let party_entity = app
            .world
            .spawn((Party::default(), Group::default(), Movement::default()))
            .id();
        let member_entity = app.world.spawn(Movement { points: 2 }).id();
        let joingroup = JoinGroup {
            group: party_entity,
            members: SmallVec::from_slice(&[member_entity]),
        };
        joingroup.write(&mut app.world);
        app
    }

    #[rstest]
    fn join_group(mut app: App) {
        let (group_entity, group) = app.world.query::<(Entity, &Group)>().single(&app.world);
        assert_eq!(group.members.len(), 1);
        let member_from_group_entity = group.members[0];

        let (member_entity, member) = app
            .world
            .query::<(Entity, &GroupMember)>()
            .single(&app.world);

        assert_eq!(member_from_group_entity, member_entity);
        assert_eq!(member.group, group_entity);
    }

    #[rstest]
    fn change_group(mut app: App) {
        let (member_entity, _) = app
            .world
            .query::<(Entity, &GroupMember)>()
            .single(&app.world);

        let new_group_entity = app.world.spawn(Group::default()).id();
        let joingroup = JoinGroup {
            group: new_group_entity,
            members: SmallVec::from_slice(&[member_entity]),
        };
        joingroup.write(&mut app.world);

        let group = app
            .world
            .query::<&Group>()
            .get(&app.world, new_group_entity)
            .unwrap();

        assert_eq!(group.members.len(), 1);
        assert_eq!(group.members[0], member_entity);

        let member = app.world.query::<&GroupMember>().single(&app.world);
        assert_eq!(member.group, new_group_entity);
    }

    #[rstest]
    fn party_movement(mut app: App) {
        let (mut movement, _member) = app
            .world
            .query::<(&mut Movement, &GroupMember)>()
            .single_mut(&mut app.world);
        movement.points = 3;

        app.update();

        let (party_movement, _party) = app.world.query::<(&Movement, &Party)>().single(&app.world);
        assert_eq!(party_movement.points, 3);
    }
}
