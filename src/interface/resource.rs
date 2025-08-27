use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Resource, Default)]
pub struct Index<T: Component>(pub Vec<Entity>, PhantomData<T>);

impl<T: Component> Index<T> {
    pub fn on_add(trigger: Trigger<OnAdd, T>, mut index: ResMut<Self>) {
        index.0.push(trigger.target())
    }

    pub fn on_remove(trigger: Trigger<OnRemove, T>, mut index: ResMut<Self>) {
        index.0.retain(|&e| e != trigger.target());
    }
}
