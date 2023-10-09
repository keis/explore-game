use super::component::*;
use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};
use smallvec::SmallVec;
use std::collections::HashSet;

pub(super) struct AddMembers {
    pub group: Entity,
    pub members: SmallVec<[Entity; 8]>,
}

pub(super) struct RemoveMembers {
    pub group: Entity,
    pub members: SmallVec<[Entity; 8]>,
}

pub trait GroupCommandsExt {
    fn add_members(&mut self, members: &[Entity]) -> &mut Self;
    fn remove_members(&mut self, members: &[Entity]) -> &mut Self;
    fn join_group(&mut self, group: Entity) -> &mut Self;
}

impl<'w, 's, 'a> GroupCommandsExt for EntityCommands<'w, 's, 'a> {
    fn add_members(&mut self, members: &[Entity]) -> &mut Self {
        let group = self.id();
        self.commands().add(AddMembers {
            group,
            members: SmallVec::from(members),
        });
        self
    }

    fn remove_members(&mut self, members: &[Entity]) -> &mut Self {
        let group = self.id();
        self.commands().add(RemoveMembers {
            group,
            members: SmallVec::from(members),
        });
        self
    }

    fn join_group(&mut self, group: Entity) -> &mut Self {
        let members = SmallVec::from_slice(&[self.id()]);
        self.commands().add(AddMembers { group, members });
        self
    }
}

impl Command for AddMembers {
    fn apply(mut self, world: &mut World) {
        let mut old = HashSet::new();
        for &member in &self.members {
            if let Some(mut group_member) = world.entity_mut(member).get_mut::<GroupMember>() {
                if group_member.group != Some(self.group) {
                    if let Some(group) = group_member.group {
                        old.insert(group);
                    }
                    group_member.group = Some(self.group);
                }
            } else {
                world.entity_mut(member).insert(GroupMember {
                    group: Some(self.group),
                });
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
    fn apply(self, world: &mut World) {
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
