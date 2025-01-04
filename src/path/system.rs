use super::{bundle::*, component::*, mesh::update_path_mesh};
use crate::scene::save::Save;
use bevy::prelude::*;
use splines::{Interpolation, Key, Spline};
use std::iter;

pub fn update_path_display(
    path_guided_query: Query<(Entity, &PathGuided), Changed<PathGuided>>,
    path_display_query: Query<(Entity, &PathDisplay, &mut Mesh3d)>,
    transform_query: Query<&Transform>,
    mut path_display_params: PathDisplayParams,
    mut commands: Commands,
) {
    for (entity, path_guided) in path_guided_query.iter() {
        if path_guided.path.is_empty() {
            if let Some((path_display_entity, _, _)) = path_display_query
                .iter()
                .find(|(_, path_display, _)| path_display.path_guided == entity)
            {
                commands.entity(path_display_entity).despawn();
            }
            continue;
        }

        let start_entity = path_guided.current().unwrap();
        let start = transform_query.get(start_entity).unwrap().translation;
        let mut prev: Vec3 = start;
        let end_entity = *path_guided.last().unwrap();
        let end = transform_query.get(end_entity).unwrap().translation;
        let path = Path {
            spline: Spline::from_iter(
                iter::once(Key::new(0.0, start, Interpolation::default()))
                    .chain(path_guided.path.iter().enumerate().map(|(idx, &entity)| {
                        let transform = transform_query.get(entity).unwrap();
                        let new: Vec3 = transform.translation;
                        let edge = prev + (new - prev) / 2.0;
                        let interpolation = Interpolation::Bezier(new);
                        prev = new;
                        Key::new(
                            (idx + 1) as f32 * (1.0 / (path_guided.path.len() + 1) as f32),
                            edge,
                            interpolation,
                        )
                    }))
                    .chain(iter::once(Key::new(1.0, end, Interpolation::default()))),
            ),
            steps: 16 * (path_guided.path.len() as u32 + 1),
            stroke: 0.05,
        };

        if let Some((_, _, mesh_handle)) = path_display_query
            .iter()
            .find(|(_, path_display, _)| path_display.path_guided == entity)
        {
            let mesh = path_display_params.0.get_mut(mesh_handle).unwrap();
            update_path_mesh(path, mesh);
        } else {
            commands.spawn((
                Name::new("Path Display"),
                Save,
                PathDisplayBundle::new(&mut path_display_params, entity, path),
            ));
        }
    }
}
