use bevy::{ecs::query::QueryFilter, prelude::*};

pub fn despawn_all<T: QueryFilter>(mut commands: Commands, cleanup_query: Query<Entity, T>) {
    for entity in &cleanup_query {
        commands.entity(entity).despawn();
    }
}
