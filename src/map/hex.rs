use bevy::{prelude::*, render::mesh::Indices, render::mesh::PrimitiveTopology};
pub const HEX_RADIUS_RATIO: f32 = 0.866_025_4;
pub const HEX_RADIUS: f32 = 1.0;

use super::HexCoord;

pub struct Hexagon {
    pub radius: f32,
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
        let (outer, inner) = (hexagon.radius, hexagon.radius * HEX_RADIUS_RATIO);

        let vertices = [
            ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.5, 0.5]),
            ([outer, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.5]),
            (
                [0.5 * outer, 0.0, -inner],
                [0.0, 1.0, 0.0],
                [0.75, 0.5 - 0.5 * HEX_RADIUS_RATIO],
            ),
            (
                [-0.5 * outer, 0.0, -inner],
                [0.0, 1.0, 0.0],
                [0.25, 0.5 - 0.5 * HEX_RADIUS_RATIO],
            ),
            ([-outer, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.5]),
            (
                [-0.5 * outer, 0.0, inner],
                [0.0, 1.0, 0.0],
                [0.25, 0.5 + 0.5 * HEX_RADIUS_RATIO],
            ),
            (
                [0.5 * outer, 0.0, inner],
                [0.0, 1.0, 0.0],
                [0.75, 0.5 + 0.5 * HEX_RADIUS_RATIO],
            ),
        ];
        let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1]);
        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}

#[derive(Resource)]
pub struct HexAssets {
    pub mesh: Handle<Mesh>,
}

pub fn insert_hex_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(HexAssets {
        mesh: meshes.add(Mesh::from(Hexagon { radius: 1.0 })),
    });
}

#[cfg(test)]
mod tests {
    use super::{coord_to_vec3, HexCoord, HEX_RADIUS_RATIO};
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
}
