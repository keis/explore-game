use bevy::{prelude::*, render::mesh::Indices, render::mesh::PrimitiveTopology};
use glam::Vec3A;
use hexasphere::{interpolation, BaseShape, Subdivided, Triangle};
use std::iter;
pub const HEX_RADIUS_RATIO: f32 = 0.866_025_4;
pub const HEX_RADIUS: f32 = 1.0;

use super::HexCoord;

pub struct Hexagon {
    pub radius: f32,
    pub subdivisions: usize,
}

pub fn coord_to_vec3(coord: HexCoord) -> Vec3 {
    let (outer, inner) = (HEX_RADIUS, HEX_RADIUS * HEX_RADIUS_RATIO);
    Vec3::new(
        ((coord.q as f32) + 0.5 * coord.r as f32) * inner * 2.0,
        0.0,
        coord.r as f32 * outer * 1.5,
    )
}

impl From<Hexagon> for Mesh {
    fn from(hexagon: Hexagon) -> Self {
        let generated = SubdividedHexagon::new(hexagon.subdivisions, |_| ());
        let indices = Indices::U32(generated.get_all_indices());
        let positions: Vec<_> = generated
            .raw_points()
            .iter()
            .map(|&p| (p * hexagon.radius).into())
            .collect::<Vec<[f32; 3]>>();
        let normals: Vec<_> = iter::repeat([0.0, 1.0, 0.0])
            .take(positions.len())
            .collect::<Vec<[f32; 3]>>();
        let uvs: Vec<_> = generated
            .raw_points()
            .iter()
            .map(|point| [point[0] * 0.5 + 0.5, point[2] * 0.5 + 0.5])
            .collect();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct HexagonBase;

impl BaseShape for HexagonBase {
    #[inline]
    fn initial_points(&self) -> Vec<Vec3A> {
        consts::INITIAL_POINTS.to_vec()
    }

    #[inline]
    fn triangles(&self) -> Box<[Triangle]> {
        consts::TRIANGLES.to_vec().into()
    }
    const EDGES: usize = consts::EDGES;

    #[inline]
    fn interpolate(&self, a: Vec3A, b: Vec3A, p: f32) -> Vec3A {
        interpolation::lerp(a, b, p)
    }

    #[inline]
    fn interpolate_half(&self, a: Vec3A, b: Vec3A) -> Vec3A {
        interpolation::lerp_half(a, b)
    }

    #[inline]
    fn interpolate_multiple(&self, a: Vec3A, b: Vec3A, indices: &[u32], points: &mut [Vec3A]) {
        interpolation::lerp_multiple(a, b, indices, points)
    }
}

pub type SubdividedHexagon<T> = Subdivided<T, HexagonBase>;

#[derive(Resource)]
pub struct HexAssets {
    pub mesh: Handle<Mesh>,
}

pub fn insert_hex_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(HexAssets {
        mesh: meshes.add(Mesh::from(Hexagon {
            radius: 1.0,
            subdivisions: 2,
        })),
    });
}

mod consts {
    const HEX_RADIUS_RATIO: f32 = 0.866_025_4;

    use glam::Vec3A;
    use hexasphere::Triangle;

    pub static INITIAL_POINTS: [Vec3A; 7] = [
        Vec3A::new(0.0, 0.0, 0.0),
        Vec3A::new(1.0, 0.0, 0.0),
        Vec3A::new(0.5, 0.0, -HEX_RADIUS_RATIO),
        Vec3A::new(-0.5, 0.0, -HEX_RADIUS_RATIO),
        Vec3A::new(-1.0, 0.0, 0.0),
        Vec3A::new(-0.5, 0.0, HEX_RADIUS_RATIO),
        Vec3A::new(0.5, 0.0, HEX_RADIUS_RATIO),
    ];

    pub const TRIANGLES: [Triangle; 6] = [
        Triangle::new(0, 1, 2, 0, 1, 2),
        Triangle::new(0, 2, 3, 2, 3, 4),
        Triangle::new(0, 3, 4, 4, 5, 6),
        Triangle::new(0, 4, 5, 6, 7, 8),
        Triangle::new(0, 5, 6, 8, 9, 10),
        Triangle::new(0, 6, 1, 10, 11, 0),
    ];

    pub const EDGES: usize = 12;
}

#[cfg(test)]
mod tests {
    use super::{coord_to_vec3, HexCoord, SubdividedHexagon, HEX_RADIUS_RATIO};
    use bevy::prelude::*;

    #[test]
    fn coord_as_vec3() {
        assert_eq!(coord_to_vec3(HexCoord::ZERO), Vec3::ZERO);
        assert_eq!(
            coord_to_vec3(HexCoord::new(1, 0)),
            Vec3::new(2.0 * HEX_RADIUS_RATIO, 0.0, 0.0)
        );
        assert_eq!(
            coord_to_vec3(HexCoord::new(2, 0)),
            Vec3::new(4.0 * HEX_RADIUS_RATIO, 0.0, 0.0)
        );
        assert_eq!(
            coord_to_vec3(HexCoord::new(0, 1)),
            Vec3::new(HEX_RADIUS_RATIO, 0.0, 1.5)
        );
        assert_eq!(
            coord_to_vec3(HexCoord::new(1, 1)),
            Vec3::new(3.0 * HEX_RADIUS_RATIO, 0.0, 1.5)
        );
    }

    #[test]
    fn base_hexagon() {
        let hexagon = SubdividedHexagon::new(0, |_| ());
        let raw_points = hexagon.raw_points();
        let indices = hexagon.get_all_indices();
        println!("points {:?} indices {:?}", raw_points, indices);
        assert_eq!(raw_points.len(), 7);
        assert_eq!(indices.len(), 18);
    }
}
