use bevy_camera::visibility::Visibility;
use bevy_ecs::{
    error::BevyError,
    prelude::*,
    template::{Template, TemplateContext},
};

use super::Fog;

/// Scene template inserting the [`Visibility`] implied by [`Fog`].
///
/// Used in scenes as `~FogVisibility`, deriving the visibility from the [`Fog`]
/// already present on the entity.
#[derive(Debug, Default)]
pub struct FogVisibility;

impl Template for FogVisibility {
    type Output = Visibility;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Visibility> {
        let fog = context
            .entity
            .get::<Fog>()
            .ok_or_else(|| BevyError::error("FogVisibility requires Fog on the entity"))?;
        Ok(Visibility::from(*fog))
    }

    fn clone_template(&self) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::template::SceneEntityReferences;

    fn build_visibility(world: &mut World, entity: Entity) -> Result<Visibility> {
        let mut entity = world.entity_mut(entity);
        let mut references = SceneEntityReferences::default();
        let mut context = TemplateContext::new(&mut entity, &mut references);
        FogVisibility.build_template(&mut context)
    }

    #[test]
    fn explored_fog_inherits_visibility() {
        let mut world = World::new();
        let entity = world
            .spawn(Fog {
                visible: true,
                explored: true,
            })
            .id();

        assert_eq!(
            build_visibility(&mut world, entity).unwrap(),
            Visibility::Inherited
        );
    }

    #[test]
    fn unexplored_fog_hides_visibility() {
        let mut world = World::new();
        let entity = world
            .spawn(Fog {
                visible: true,
                explored: false,
            })
            .id();

        assert_eq!(
            build_visibility(&mut world, entity).unwrap(),
            Visibility::Hidden
        );
    }

    #[test]
    fn missing_fog_fails_to_build() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        assert!(build_visibility(&mut world, entity).is_err());
    }
}
