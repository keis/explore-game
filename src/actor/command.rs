use super::{component::*, event::*};
use bevy::{ecs::system::EntityCommands, ecs::world::Command, prelude::*};
use smallvec::SmallVec;

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

impl GroupCommandsExt for EntityCommands<'_> {
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

fn update_group_member(world: &mut World, member: Entity, new_group: Entity) -> Option<Entity> {
    let mut member = world.entity_mut(member);
    if let Some(mut group_member) = member.get_mut::<Group>() {
        let previous = group_member.get();
        *group_member = Group(new_group);
        Some(previous)
    } else {
        member.insert(Group(new_group));
        None
    }
}

impl Command for AddMembers {
    fn apply(mut self, world: &mut World) {
        for &member in &self.members {
            let previous_group = update_group_member(world, member, self.group);
            if let Some(previous_group) = previous_group {
                if previous_group == self.group {
                    continue;
                }
                if let Some(mut members) = world.entity_mut(previous_group).get_mut::<Members>() {
                    members.0.retain(|m| *m != member);
                }
                world.send_event(GroupEvent::MemberRemoved {
                    group: previous_group,
                    member,
                });
            }
            world.send_event(GroupEvent::MemberAdded {
                group: self.group,
                member,
            });
        }

        let mut group_entity = world.entity_mut(self.group);
        if let Some(mut members) = group_entity.get_mut::<Members>() {
            members.0.append(&mut self.members);
        }
    }
}

impl Command for RemoveMembers {
    fn apply(self, world: &mut World) {
        let mut group_entity = world.entity_mut(self.group);
        if let Some(mut members) = group_entity.get_mut::<Members>() {
            members.0.retain(|member| !self.members.contains(member));
        }
        for member in self.members {
            world.entity_mut(member).remove::<Parent>();
            world.send_event(GroupEvent::MemberRemoved {
                group: self.group,
                member,
            });
        }
    }
}
