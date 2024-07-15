use bevy_render::{
    mesh::{Indices, Meshable, PrimitiveTopology},
    prelude::*,
    render_asset::RenderAssetUsages,
};
use glam::Vec3A;
use hexasphere::{interpolation, BaseShape, Subdivided, Triangle};
use std::iter;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hexagon {
    pub radius: f32,
}

impl Default for Hexagon {
    fn default() -> Self {
        Self { radius: 1.0 }
    }
}

impl Hexagon {
    #[inline(always)]
    pub const fn new(radius: f32) -> Self {
        Self { radius }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HexagonMeshBuilder {
    pub hexagon: Hexagon,
    pub subdivisions: usize,
}

impl HexagonMeshBuilder {
    #[inline]
    pub const fn subdivisions(mut self, subdivisions: usize) -> Self {
        self.subdivisions = subdivisions;
        self
    }
}

impl MeshBuilder for HexagonMeshBuilder {
    fn build(&self) -> Mesh {
        let generated = SubdividedHexagon::new(self.subdivisions, |_| ());
        let indices = Indices::U32(generated.get_all_indices());
        let positions: Vec<_> = generated
            .raw_points()
            .iter()
            .map(|&p| (p * self.hexagon.radius).into())
            .collect::<Vec<[f32; 3]>>();
        let normals: Vec<_> = iter::repeat([0.0, 1.0, 0.0])
            .take(positions.len())
            .collect::<Vec<[f32; 3]>>();
        let uvs: Vec<_> = generated
            .raw_points()
            .iter()
            .map(|point| [point[0] * 0.5 + 0.5, point[2] * 0.5 + 0.5])
            .collect();

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_indices(indices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}

impl Meshable for Hexagon {
    type Output = HexagonMeshBuilder;

    fn mesh(&self) -> Self::Output {
        HexagonMeshBuilder {
            hexagon: *self,
            ..Default::default()
        }
    }
}

impl From<Hexagon> for Mesh {
    fn from(hexagon: Hexagon) -> Self {
        hexagon.mesh().into()
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

mod consts {
    const HEX_RADIUS_RATIO: f32 = 0.866_025_4;

    use glam::Vec3A;
    use hexasphere::Triangle;

    pub static INITIAL_POINTS: [Vec3A; 7] = [
        Vec3A::new(0.0, 0.0, 0.0),
        Vec3A::new(-HEX_RADIUS_RATIO, 0.0, 0.5),
        Vec3A::new(0.0, 0.0, 1.0),
        Vec3A::new(HEX_RADIUS_RATIO, 0.0, 0.5),
        Vec3A::new(HEX_RADIUS_RATIO, 0.0, -0.5),
        Vec3A::new(0.0, 0.0, -1.0),
        Vec3A::new(-HEX_RADIUS_RATIO, 0.0, -0.5),
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
    use super::{Hexagon, SubdividedHexagon};
    use bevy_render::mesh::{Mesh, MeshBuilder, Meshable};

    #[test]
    fn base_hexagon() {
        let hexagon = SubdividedHexagon::new(0, |_| ());
        let raw_points = hexagon.raw_points();
        let indices = hexagon.get_all_indices();
        println!("points {:?} indices {:?}", raw_points, indices);
        assert_eq!(raw_points.len(), 7);
        assert_eq!(indices.len(), 18);
    }

    #[test]
    fn default_mesh() {
        let hexagon = Hexagon::default();
        let mesh: Mesh = hexagon.mesh().build();
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
    }
}
