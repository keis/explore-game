use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};
use core::slice;
use smallvec::SmallVec;

#[derive(Component)]
pub struct DataBindings(SmallVec<[Entity; 8]>);

impl<'a> IntoIterator for &'a DataBindings {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = slice::Iter<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub struct Bind {
    source: Entity,
    sink: Entity,
}

impl Command for Bind {
    fn apply(self, world: &mut World) {
        let mut source_entity = world.entity_mut(self.source);
        if let Some(mut data_bindings) = source_entity.get_mut::<DataBindings>() {
            data_bindings.0.push(self.sink);
        } else {
            source_entity.insert(DataBindings(SmallVec::from_slice(&[self.sink])));
        }
    }
}

pub trait DataBindingExt {
    fn bind_to(&mut self, source: Entity) -> &mut Self;
}

impl<'w, 's, 'a> DataBindingExt for EntityCommands<'w, 's, 'a> {
    fn bind_to(&mut self, source: Entity) -> &mut Self {
        let sink = self.id();
        self.commands().add(Bind { source, sink });
        self
    }
}
