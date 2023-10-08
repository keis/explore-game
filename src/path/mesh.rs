use super::component::*;
use bevy::{
    math::Vec3Swizzles, prelude::*, render::mesh::Indices, render::mesh::PrimitiveTopology,
};
use itertools::Itertools;

pub fn update_path_mesh(path: Path, mesh: &mut Mesh) {
    let step = 1.0 / path.steps as f32;
    let mut points =
        (0..path.steps + 1).map(|i| path.spline.clamped_sample(i as f32 * step).unwrap());

    // Compute orthogonal vector used for width for each point
    let mut segments = Vec::new();
    let mut lastpoint = points.next().unwrap();
    segments.push((lastpoint, Vec2::ZERO));
    for point in points {
        segments.push((point, (point - lastpoint).zx().normalize()));
        lastpoint = point;
    }
    segments.push((Vec3::ZERO, Vec2::ZERO));

    let vertices: Vec<_> = segments
        .iter()
        // Average the width for internal segments and apply stroke width
        .tuple_windows()
        .map(|((point, width), (_, next_width))| {
            (*point, (*width + *next_width).normalize() * path.stroke)
        })
        // Create vertices for each segment
        .flat_map(|(point, width)| {
            vec![
                (
                    [point.x + width.x, point.y, point.z - width.y],
                    [0.0, 1.0, 0.0],
                    [0.0, 1.0],
                ),
                (
                    [point.x - width.x, point.y, point.z + width.y],
                    [0.0, 1.0, 0.0],
                    [1.0, 1.0],
                ),
            ]
        })
        .collect();

    let indextmpl = [0, 1, 3, 0, 3, 2];
    let indices = Indices::U32(
        (0..path.steps)
            .map(|idx| idx * 2)
            .flat_map(|a| indextmpl.iter().map(move |b| a + b))
            .collect(),
    );

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
}

pub fn empty_path_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList)
}

impl From<Path> for Mesh {
    fn from(path: Path) -> Mesh {
        let mut mesh = empty_path_mesh();
        update_path_mesh(path, &mut mesh);
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::Path;
    use bevy::{prelude::*, render::mesh::VertexAttributeValues};
    use splines::{Interpolation, Key, Spline};

    #[test]
    fn splines() {
        let spline = Spline::from_vec(vec![
            Key::new(
                0.0,
                Vec3::new(0.0, 0.0, 0.0),
                Interpolation::Bezier(Vec3::new(1.5, 0.0, -0.5)),
            ),
            Key::new(1.0, Vec3::new(1.0, 0.0, 1.0), Interpolation::default()),
        ]);

        assert_eq!(spline.sample(0.5), Some(Vec3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn right_angle() {
        let path = Path {
            spline: Spline::from_vec(vec![
                Key::new(
                    0.0,
                    Vec3::new(0.0, 0.0, 0.0),
                    Interpolation::Bezier(Vec3::new(1.5, 0.0, -0.5)),
                ),
                Key::new(1.0, Vec3::new(1.0, 0.0, 1.0), Interpolation::default()),
            ]),
            steps: 2,
            stroke: 0.1,
        };
        let mesh: Mesh = path.into();
        if let VertexAttributeValues::Float32x3(positions) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        {
            println!("vertices: {:?}", positions);
            // Around origin
            assert!(Vec3::from_array(positions[0]).abs_diff_eq(Vec3::new(0.0, 0.0, -0.1), 0.01));
            assert!(Vec3::from_array(positions[1]).abs_diff_eq(Vec3::new(0.0, 0.0, 0.1), 0.01));

            // Around joint
            assert!(Vec3::from_array(positions[2]).abs_diff_eq(Vec3::new(1.07, 0.0, -0.07), 0.01));
            assert!(Vec3::from_array(positions[3]).abs_diff_eq(Vec3::new(0.92, 0.0, 0.07), 0.01));

            // Around end
            assert!(Vec3::from_array(positions[4]).abs_diff_eq(Vec3::new(1.1, 0.0, 1.0), 0.01));
            assert!(Vec3::from_array(positions[5]).abs_diff_eq(Vec3::new(0.9, 0.0, 1.0), 0.01));
        } else {
            assert_eq!(0, 1);
        }

        let indices = mesh.indices().unwrap();
        println!("indices: {:?}", indices);
        assert_eq!(12, indices.len());
    }
}
