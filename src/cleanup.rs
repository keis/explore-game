use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*};

pub fn despawn_all<T: ReadOnlyWorldQuery>(mut commands: Commands, cleanup_query: Query<Entity, T>) {
    for entity in &cleanup_query {
        commands.entity(entity).despawn_recursive();
    }
}
