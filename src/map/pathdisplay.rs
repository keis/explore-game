use super::{coord_to_vec3, PathGuided};
use crate::path::{path_mesh, update_path_mesh, Path};
use bevy::{pbr::NotShadowCaster, prelude::*};
use splines::{Interpolation, Key, Spline};
use std::iter;

#[derive(Component)]
pub struct PathDisplay {
    pub path_guided: Entity,
}

pub fn update_path_display(
    path_guided_query: Query<(Entity, &PathGuided), Changed<PathGuided>>,
    path_display_query: Query<(Entity, &PathDisplay, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
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

        let start = path_guided.current().unwrap();
        let mut prev = coord_to_vec3(start);
        let end = *path_guided.last().unwrap();
        let path = Path {
            spline: Spline::from_iter(
                iter::once(Key::new(
                    0.0,
                    coord_to_vec3(start),
                    Interpolation::default(),
                ))
                .chain(path_guided.path.iter().enumerate().map(|(idx, &pos)| {
                    let new = coord_to_vec3(pos);
                    let edge = prev + (new - prev) / 2.0;
                    let interpolation = Interpolation::Bezier(new);
                    prev = new;
                    Key::new(
                        (idx + 1) as f32 * (1.0 / (path_guided.path.len() + 1) as f32),
                        edge,
                        interpolation,
                    )
                }))
                .chain(iter::once(Key::new(
                    1.0,
                    coord_to_vec3(end),
                    Interpolation::default(),
                ))),
            ),
            steps: 16 * (path_guided.path.len() as u32 + 1),
            stroke: 0.05,
        };

        if let Some((_, _, mesh_handle)) = path_display_query
            .iter()
            .find(|(_, path_display, _)| path_display.path_guided == entity)
        {
            let mesh = meshes.get_mut(mesh_handle).unwrap();
            update_path_mesh(path, mesh);
        } else {
            commands.spawn((
                PathDisplay {
                    path_guided: entity,
                },
                PbrBundle {
                    mesh: meshes.add(path_mesh(path)),
                    material: standard_materials.add(Color::rgba(0.8, 0.8, 0.8, 0.6).into()),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                    ..default()
                },
                NotShadowCaster,
            ));
        }
    }
}
