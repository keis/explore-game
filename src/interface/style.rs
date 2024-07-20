use bevy::{
    ecs::{system::EntityCommands, world::Command},
    prelude::*,
};
pub use bevy_mod_stylebuilder::*;

struct ApplyStyle<S: StyleTuple + 'static> {
    entity: Entity,
    style: S,
}

impl<S: StyleTuple + 'static> Command for ApplyStyle<S> {
    fn apply(self, world: &mut World) {
        let mut entity = world.entity_mut(self.entity);
        let style = if let Some(style) = entity.get::<Style>() {
            style.clone()
        } else {
            Style::default()
        };
        let mut builder = StyleBuilder::new(&mut entity, style);
        self.style.apply(&mut builder);
        builder.finish()
    }
}

pub trait StyleCommandsExt {
    fn with_style<S: StyleTuple + 'static>(&mut self, style: S) -> &mut Self;
}

impl<'a> StyleCommandsExt for EntityCommands<'a> {
    fn with_style<S: StyleTuple + 'static>(&mut self, style: S) -> &mut Self {
        let entity = self.id();
        self.commands().add(ApplyStyle { entity, style });
        self
    }
}
