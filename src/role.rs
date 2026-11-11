//! Roles attach presentation ("fluff") to simulation entities.
//!
//! Every successful attach also inserts [`AssumedRoles`]. Fluff systems detect
//! unfluffed entities using `Without<AssumedRoles>`.
use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
    scene::{EntityWorldMutSceneExt, Scene},
};

use crate::ExplError;

/// Marker inserted by every successful [`Role`] attach.
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
        let scene = self.role.scene(entity.world());
        let result = scene.and_then(|scene| entity.apply_scene(scene).map_err(ExplError::from));
        if let Err(err) = result {
            ::tracing::warn!(
                role = core::any::type_name::<R>(),
                entity = ?self.entity,
                error = ?err,
                "failed to attach role"
            );
            return;
        }
        entity.insert(AssumedRoles);
    }
}

pub trait RoleCommandsExt {
    fn attach_role<R: Role>(&mut self, role: R) -> &mut Self;
}

impl RoleCommandsExt for EntityCommands<'_> {
    fn attach_role<R: Role>(&mut self, role: R) -> &mut Self {
        let entity = self.id();
        self.commands().queue(AttachRole { entity, role });
        self
    }
}

/// Presentation attached to a simulation entity.
pub trait Role: Send + Sync + 'static {
    /// Builds the presentation scene from the role and the [`World`] it is
    /// attached in.
    ///
    /// The scene is returned instead of applied so roles can resolve codex
    /// entries and other resources first.
    fn scene(self, world: &World) -> Result<impl Scene, ExplError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::AssetPlugin,
        ecs::{
            error::BevyError,
            template::{Template, TemplateContext},
        },
        scene::{bsn, ScenePlugin},
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin));
        app
    }

    struct EmptyRole;

    impl Role for EmptyRole {
        fn scene(self, _world: &World) -> Result<impl Scene, ExplError> {
            Ok(bsn! { Name::new("Empty") })
        }
    }

    #[derive(Default)]
    struct FailingTemplate;

    impl Template for FailingTemplate {
        type Output = Name;

        fn build_template(&self, _context: &mut TemplateContext) -> Result<Name> {
            Err(BevyError::error("template failure"))
        }

        fn clone_template(&self) -> Self {
            Self
        }
    }

    struct FailingRole;

    impl Role for FailingRole {
        fn scene(self, _world: &World) -> Result<impl Scene, ExplError> {
            Ok(bsn! { ~FailingTemplate })
        }
    }

    #[test]
    fn successful_attach_marks_assumed_roles() {
        let mut app = test_app();
        let entity = app.world_mut().spawn_empty().id();

        AttachRole {
            entity,
            role: EmptyRole,
        }
        .apply(app.world_mut());

        assert!(app.world().entity(entity).contains::<AssumedRoles>());
    }

    #[test]
    fn failed_attach_does_not_mark_assumed_roles() {
        let mut app = test_app();
        let entity = app.world_mut().spawn_empty().id();

        AttachRole {
            entity,
            role: FailingRole,
        }
        .apply(app.world_mut());

        assert!(!app.world().entity(entity).contains::<AssumedRoles>());
    }
}
