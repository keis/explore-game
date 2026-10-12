use super::{bundle::*, component::*, mesh::update_path_mesh};
use crate::scene::save::Save;
use bevy::{
    math::cubic_splines::{CubicBezier, CubicGenerator},
    prelude::*,
};
use itertools::Itertools;

/// Cubic Bézier control points for one path segment.
///
/// The curve is a visual ribbon only, so segments prioritize looking smooth
/// over exact arc-length parametrization:
///
/// - A missing *start* handle always yields a straight line; a handle on the
///   far endpoint alone is ignored. In practice only the lead segment from the
///   moving unit has no start handle, giving a straight pull-off.
/// - Segments between two handled midpoints use mirrored tangents, keeping the
///   joint C1-continuous in direction.
/// - The final segment into the plain endpoint is a degree-elevated quadratic,
///   easing into the destination.
fn bezier_segment(
    a: Vec3,
    start_handle: Option<Vec3>,
    b: Vec3,
    end_handle: Option<Vec3>,
) -> [Vec3; 4] {
    match (start_handle, end_handle) {
        (None, None) | (None, Some(_)) => {
            let delta = (b - a) / 3.0;
            [a, a + delta, b - delta, b]
        }
        (Some(u), Some(v)) => [a, u, b + b - v, b],
        (Some(u), None) => [a, (a + 2.0 * u) / 3.0, (2.0 * u + b) / 3.0, b],
    }
}

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
        let end_entity = *path_guided.last().unwrap();
        let end = transform_query.get(end_entity).unwrap().translation;

        // Path control points with optional curve handles. The first and last
        // points are plain endpoints; midpoints carry the next waypoint as
        // handle, giving eased corners with a straight pull-off from the unit.
        // Keys stream straight into Bézier control points with no buffering.
        let segments = std::iter::once((start, None))
            .chain(path_guided.path.iter().scan(start, |prev, &entity| {
                let new: Vec3 = transform_query.get(entity).unwrap().translation;
                let edge = *prev + (new - *prev) / 2.0;
                *prev = new;
                Some((edge, Some(new)))
            }))
            .chain(std::iter::once((end, None)))
            .tuple_windows()
            .map(|((a, start_handle), (b, end_handle))| {
                bezier_segment(a, start_handle, b, end_handle)
            });
        let Ok(spline) = CubicBezier::new(segments).to_curve() else {
            continue;
        };
        let path = Path {
            spline,
            steps: 16 * (path_guided.path.len() as u32 + 1),
            stroke: 0.05,
        };

        if let Some((_, _, mesh_handle)) = path_display_query
            .iter()
            .find(|(_, path_display, _)| path_display.path_guided == entity)
        {
            let mesh = path_display_params.0.get_mut(mesh_handle).unwrap();
            update_path_mesh(path, mesh.into_inner());
        } else {
            commands.spawn((
                Name::new("Path Display"),
                Save,
                PathDisplayBundle::new(&mut path_display_params, entity, path),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::bezier_segment;
    use bevy::{
        math::cubic_splines::{CubicBezier, CubicGenerator},
        prelude::*,
    };

    fn keys_for_two_waypoints() -> Vec<(Vec3, Option<Vec3>)> {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let w0 = Vec3::new(2.0, 0.0, 0.0);
        let w1 = Vec3::new(2.0, 0.0, 2.0);
        let end = Vec3::new(4.0, 0.0, 2.0);
        let edge0 = start + (w0 - start) / 2.0;
        let edge1 = w0 + (w1 - w0) / 2.0;
        vec![
            (start, None),
            (edge0, Some(w0)),
            (edge1, Some(w1)),
            (end, None),
        ]
    }

    #[test]
    fn straight_without_start_handle_ignores_end_handle() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(3.0, 0.0, 0.0);
        let delta = (b - a) / 3.0;
        let expected = [a, a + delta, b - delta, b];
        assert_eq!(bezier_segment(a, None, b, None), expected);
        assert_eq!(
            bezier_segment(a, None, b, Some(Vec3::new(9.0, 0.0, 9.0))),
            expected
        );
    }

    #[test]
    fn middle_segment_mirrors_end_tangent() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let u = Vec3::new(2.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 1.0);
        let v = Vec3::new(2.0, 0.0, 2.0);
        assert_eq!(bezier_segment(a, Some(u), b, Some(v)), [a, u, b + b - v, b]);
    }

    #[test]
    fn tail_segment_elevates_quadratic() {
        let a = Vec3::new(2.0, 0.0, 1.0);
        let u = Vec3::new(2.0, 0.0, 2.0);
        let b = Vec3::new(4.0, 0.0, 2.0);
        assert_eq!(
            bezier_segment(a, Some(u), b, None),
            [a, (a + 2.0 * u) / 3.0, (2.0 * u + b) / 3.0, b]
        );
    }

    #[test]
    fn multi_segment_curve_holds_visual_shape() {
        let keys = keys_for_two_waypoints();
        let segments: Vec<[Vec3; 4]> = keys
            .windows(2)
            .map(|window| {
                let (a, start_handle) = window[0];
                let (b, end_handle) = window[1];
                bezier_segment(a, start_handle, b, end_handle)
            })
            .collect();
        assert_eq!(segments.len(), 3);

        // C0 continuity: segments share endpoints.
        assert_eq!(segments[0][3], segments[1][0]);
        assert_eq!(segments[1][3], segments[2][0]);

        // Interior joint keeps its direction through the mirrored tangent.
        // (Checked on the control points before the curve takes ownership.)
        let incoming = segments[1][3] - segments[1][2];
        let outgoing = segments[2][1] - segments[2][0];
        assert!(incoming.normalize().abs_diff_eq(outgoing.normalize(), 1e-5),);

        let curve = CubicBezier::new(segments).to_curve().unwrap();
        let (start, _) = keys[0];
        let (edge0, _) = keys[1];
        let (edge1, _) = keys[2];
        let (end, _) = keys[3];

        // Curve passes through every key at integer knots.
        for (i, key) in [start, edge0, edge1, end].iter().enumerate() {
            assert!(
                curve.position(i as f32).abs_diff_eq(*key, 1e-5),
                "knot {i} should sit on its key"
            );
        }

        // Lead segment is a straight pull-off.
        assert!(curve.position(0.5).abs_diff_eq((start + edge0) / 2.0, 1e-5),);
    }
}
