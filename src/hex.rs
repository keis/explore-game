use bevy::{prelude::*, render::mesh::Indices, render::mesh::PrimitiveTopology};
use pathfinding::prelude::astar;

pub const HEX_RADIUS_RATIO: f32 = 0.866025404;

pub struct Hexagon {
    pub radius: f32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct HexCoord {
    pub q: isize,
    pub r: isize,
}

impl HexCoord {
    pub fn new(q: isize, r: isize) -> Self {
        HexCoord { q, r }
    }

    pub fn as_vec3(&self, radius: f32) -> Vec3 {
        let (outer, inner) = (radius, radius * HEX_RADIUS_RATIO);
        Vec3::new(
            self.r as f32 * outer * 1.5,
            0.0,
            ((self.q as f32) + 0.5 * self.r as f32) * inner * 2.0,
        )
    }

    pub fn distance(&self, other: &Self) -> usize {
        self.q.abs_diff(other.q) + self.r.abs_diff(other.r)
    }

    pub fn neighbours(&self) -> Vec<Self> {
        vec![
            Self::new(self.q + 1, self.r),
            Self::new(self.q, self.r + 1),
            Self::new(self.q - 1, self.r + 1),
            Self::new(self.q - 1, self.r),
            Self::new(self.q, self.r - 1),
            Self::new(self.q + 1, self.r - 1),
        ]
    }

    pub fn successors(&self) -> Vec<(Self, u32)> {
        self.neighbours().into_iter().map(|p| (p, 1)).collect()
    }
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

#[derive(Copy, Clone)]
pub struct MapLayout {
    pub width: isize,
    pub height: isize,
}

impl MapLayout {
    pub fn size(self) -> usize {
        (self.width * self.height).try_into().unwrap()
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = HexCoord> + '_ {
        MapLayoutIterator { layout: self, i: 0 }
    }

    pub fn offset(&self, position: HexCoord) -> Option<usize> {
        usize::try_from(position.r * self.width + position.q + position.r / 2)
            .ok()
            .filter(|o| o < &self.size())
    }
}

pub struct Map {
    tiles: Vec<Option<Entity>>,
    pub layout: MapLayout,
}

impl Map {
    pub fn new(layout: MapLayout) -> Self {
        Self {
            layout,
            tiles: vec![None; layout.size()],
        }
    }

    pub fn set(&mut self, position: HexCoord, entity: Option<Entity>) {
        if let Some(offset) = self.layout.offset(position) {
            self.tiles[offset] = entity;
        }
    }

    pub fn get(&self, position: HexCoord) -> Option<Entity> {
        self.layout
            .offset(position)
            .and_then(|offset| self.tiles[offset])
    }
}

pub struct MapLayoutIterator<'a> {
    layout: &'a MapLayout,
    i: isize,
}

impl<'a> Iterator for MapLayoutIterator<'a> {
    type Item = HexCoord;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.i / self.layout.width;
        let q = self.i % self.layout.width - r / 2;

        if r >= self.layout.height {
            return None;
        }

        self.i += 1;
        Some(HexCoord::new(q, r))
    }
}

pub fn find_path(
    start: HexCoord,
    goal: HexCoord,
    is_walkable: &impl Fn(&HexCoord) -> bool,
) -> Option<(Vec<HexCoord>, u32)> {
    astar(
        &start,
        |p| {
            p.neighbours()
                .into_iter()
                .filter(is_walkable)
                .map(|p| (p, 1))
                .collect::<Vec<(HexCoord, u32)>>()
        },
        |p| p.distance(&goal).try_into().unwrap(),
        |p| *p == goal,
    )
}

#[cfg(test)]
mod tests {
    use crate::hex::{find_path, HexCoord, MapLayout, HEX_RADIUS_RATIO};
    use bevy::prelude::*;

    #[test]
    fn coord_as_vec3() {
        let radius = 1.0;
        assert_eq!(HexCoord::new(0, 0).as_vec3(radius), Vec3::ZERO);
        assert_eq!(
            HexCoord::new(1, 0).as_vec3(radius),
            Vec3::new(0.0, 0.0, 2.0 * HEX_RADIUS_RATIO)
        );
        assert_eq!(
            HexCoord::new(2, 0).as_vec3(radius),
            Vec3::new(0.0, 0.0, 4.0 * HEX_RADIUS_RATIO)
        );
        assert_eq!(
            HexCoord::new(0, 1).as_vec3(radius),
            Vec3::new(1.5, 0.0, HEX_RADIUS_RATIO)
        );
        assert_eq!(
            HexCoord::new(1, 1).as_vec3(radius),
            Vec3::new(1.5, 0.0, 3.0 * HEX_RADIUS_RATIO)
        );
    }

    #[test]
    fn pathfinding_neighbour() {
        let start = HexCoord::new(2, 4);
        let goal = HexCoord::new(2, 3);

        let result = find_path(start, goal, &|_| true);
        println!("neigbours {:?}", start.neighbours());
        println!("path {:?}", result);
        assert_eq!(result.expect("no path found").1, 1);
    }

    #[test]
    fn pathfinding() {
        let start = HexCoord::new(0, 0);
        let goal = HexCoord::new(4, 2);

        let result = find_path(start, goal, &|_| true);
        println!("path {:?}", result);
        assert_eq!(result.expect("no path found").1, 6);
    }

    #[test]
    fn map_iter() {
        let layout = MapLayout {
            width: 3,
            height: 3,
        };
        let coords: Vec<HexCoord> = layout.iter().collect();

        println!("coords {:?}", coords);
        assert_eq!(coords.len(), 9);
        assert_eq!(coords[0], HexCoord::new(0, 0));
        assert_eq!(layout.offset(HexCoord::new(0, 0)).expect("in bounds"), 0);
        assert_eq!(coords[1], HexCoord::new(1, 0));
        assert_eq!(coords[2], HexCoord::new(2, 0));
        assert_eq!(coords[3], HexCoord::new(0, 1));
        assert_eq!(coords[4], HexCoord::new(1, 1));
        assert_eq!(coords[5], HexCoord::new(2, 1));
        assert_eq!(coords[6], HexCoord::new(-1, 2));
        assert_eq!(layout.offset(HexCoord::new(-1, 2)).expect("in bounds"), 6);
        assert_eq!(coords[7], HexCoord::new(0, 2));
        assert_eq!(coords[8], HexCoord::new(1, 2));
        assert_eq!(layout.offset(HexCoord::new(1, 2)).expect("in bounds"), 8);
    }
}
