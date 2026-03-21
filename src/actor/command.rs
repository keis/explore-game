use super::component::*;
use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};
use smallvec::SmallVec;

pub(super) struct AddMembers {
    pub group: Entity,
    pub members: SmallVec<[Entity; 8]>,
}

pub(super) struct RemoveMembers {
    pub members: SmallVec<[Entity; 8]>,
}

pub trait GroupCommandsExt {
    fn add_members(&mut self, members: &[Entity]) -> &mut Self;
    fn remove_members(&mut self, members: &[Entity]) -> &mut Self;
    fn join_group(&mut self, group: Entity) -> &mut Self;
}

impl GroupCommandsExt for EntityCommands<'_> {
    fn add_members(&mut self, members: &[Entity]) -> &mut Self {
        let group = self.id();
        self.commands().queue(AddMembers {
            group,
            members: SmallVec::from(members),
        });
        self
    }

    fn remove_members(&mut self, members: &[Entity]) -> &mut Self {
        self.commands().queue(RemoveMembers {
            members: SmallVec::from(members),
        });
        self
    }

    fn join_group(&mut self, group: Entity) -> &mut Self {
        let members = SmallVec::from_slice(&[self.id()]);
        self.commands().queue(AddMembers { group, members });
        self
    }
}

fn update_group_member(world: &mut World, member: Entity, new_group: Entity) -> Option<Entity> {
    let mut member = world.entity_mut(member);
    if let Some(group_member) = member.get::<MemberOf>() {
        let previous = group_member.get();
        member.insert(MemberOf(new_group));
        Some(previous)
    } else {
        member.insert(MemberOf(new_group));
        None
    }
}

impl Command for AddMembers {
    fn apply(self, world: &mut World) {
        for &member in &self.members {
            let previous_group = update_group_member(world, member, self.group);
            if previous_group == Some(self.group) {
                continue;
            }
        }
    }
}

impl Command for RemoveMembers {
    fn apply(self, world: &mut World) {
        for member in self.members {
            world.entity_mut(member).remove::<MemberOf>();
        }
    }
}
