use bevy::{
    ecs::{system::EntityCommands, world::Command},
    prelude::*,
};

struct AttachRole<R: Role> {
    entity: Entity,
    role: R,
}

impl<R: Role> Command for AttachRole<R> {
    fn apply(self, world: &mut World) {
        let mut entity = world.entity_mut(self.entity);
        self.role.attach(&mut entity);
    }
}

pub trait RoleCommandsExt {
    fn attach_role<R: Role>(&mut self, role: R) -> &mut Self;
}

impl<'a> RoleCommandsExt for EntityCommands<'a> {
    fn attach_role<R: Role + std::marker::Send>(&mut self, role: R) -> &mut Self {
        let entity = self.id();
        self.commands().add(AttachRole { entity, role });
        self
    }
}

pub trait Role: std::marker::Send + 'static {
    fn attach(self, entity: &mut EntityWorldMut);
}
