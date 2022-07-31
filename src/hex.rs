use bevy::{prelude::*, render::mesh::Indices, render::mesh::PrimitiveTopology};

pub const HEX_RADIUS_RATIO: f32 = 0.866025404;

pub struct Hexagon {
    pub radius: f32,
}

impl From<Hexagon> for Mesh {
    fn from(hexagon: Hexagon) -> Self {
        let (outer, inner) = (hexagon.radius, hexagon.radius * HEX_RADIUS_RATIO);

        let vertices = [
            ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([outer, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([0.5 * outer, 0.0, -inner], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([-0.5 * outer, 0.0, -inner], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([-outer, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([-0.5 * outer, 0.0, inner], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([0.5 * outer, 0.0, inner], [0.0, 1.0, 0.0], [0.0, 0.0]),
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
