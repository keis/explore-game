//! Roles attach presentation ("fluff") to simulation entities.
//!
//! Every `attach_role` also inserts [`AssumedRoles`]. Fluff systems detect
//! unfluffed entities using `Without<AssumedRoles>`.
use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};

/// Marker inserted by every [`Role`] attach.
///
/// Deliberately *not* in the save filter so restored entities can get the
/// roles attached fresh.
#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct AssumedRoles;

struct AttachRole<R: Role> {
    entity: Entity,
    role: R,
}

impl<R: Role> Command for AttachRole<R> {
    type Out = ();

    fn apply(self, world: &mut World) {
        let mut entity = world.entity_mut(self.entity);
        self.role.attach(&mut entity);
        entity.insert(AssumedRoles);
    }
}

pub trait RoleCommandsExt {
    fn attach_role<R: Role>(&mut self, role: R) -> &mut Self;
}

impl RoleCommandsExt for EntityCommands<'_> {
    fn attach_role<R: Role + std::marker::Send>(&mut self, role: R) -> &mut Self {
        let entity = self.id();
        self.commands().queue(AttachRole { entity, role });
        self
    }
}

pub trait Role: std::marker::Send + 'static {
    fn attach(self, entity: &mut EntityWorldMut);
}
