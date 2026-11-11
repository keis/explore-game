//! Shared scene-template machinery for materials.
use bevy::{
    ecs::{
        error::BevyError,
        template::{Template, TemplateContext},
    },
    prelude::*,
};
use core::{any::type_name, marker::PhantomData};

/// Builds a material from a scene template.
pub trait MaterialFromTemplate: Material + Sized {
    /// Data passed from the role into the template.
    type Input: Clone + Default + Send + Sync + 'static;

    /// Builds the material, using `context` for entity and resource access.
    fn from_template(input: Self::Input, context: &mut TemplateContext) -> Result<Self>;
}

/// Scene template inserting a [`MaterialFromTemplate`] as a [`MeshMaterial3d`].
pub struct MaterialTemplate<M: MaterialFromTemplate> {
    /// The role-provided input.
    pub input: M::Input,
    _marker: PhantomData<fn() -> M>,
}

impl<M: MaterialFromTemplate> Default for MaterialTemplate<M> {
    fn default() -> Self {
        Self {
            input: M::Input::default(),
            _marker: PhantomData,
        }
    }
}

impl<M: MaterialFromTemplate> Template for MaterialTemplate<M> {
    type Output = MeshMaterial3d<M>;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Self::Output> {
        let material = M::from_template(self.input.clone(), context)?;
        let handle = context.resource_mut::<Assets<M>>().add(material);
        Ok(MeshMaterial3d(handle))
    }

    fn clone_template(&self) -> Self {
        Self {
            input: self.input.clone(),
            _marker: PhantomData,
        }
    }
}

/// Reads a required [`Component`] from the entity the template is applied to.
pub fn required_component<'a, T: Component>(context: &'a TemplateContext<'_, '_>) -> Result<&'a T> {
    context.entity.get::<T>().ok_or_else(|| {
        BevyError::error(format!(
            "template requires {} on the entity",
            type_name::<T>()
        ))
    })
}
