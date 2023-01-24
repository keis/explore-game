use crate::hexgrid::layout::HexagonalGridLayout;
use crate::hexgrid::{Grid, GridLayout, HexCoord, Transform, TransformMatrix};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Index;

/// A Tile acts as a lense into a grid where a possibly rotated or reflected subsection can be accessed.
#[derive(Copy, Clone, Debug)]
pub struct Tile<'a, Layout: GridLayout, Item> {
    grid: &'a Grid<Layout, Item>,
    offset: HexCoord,
    pub layout: Layout,
    // Note that transform is applied to coordinates so effectivly the tile is rotated in the *opposite*
    // direction
    transform: &'a TransformMatrix,
}

impl<Layout: GridLayout, Item: Copy + PartialEq> Tile<'_, Layout, Item> {
    pub fn iter(&self) -> impl '_ + Iterator<Item = Item> {
        self.layout.iter().map(|coord| self[coord])
    }

    pub fn compatible_with(&self, other: &Tile<Layout, Item>, offset: HexCoord) -> bool {
        if self.layout != other.layout {
            return false;
        }

        self.layout.iter().all(|coord| {
            !self.layout.contains(coord - offset) || self[coord] == other[coord - offset]
        })
    }
}

impl<'a, Layout: GridLayout, Item> Index<HexCoord> for Tile<'a, Layout, Item> {
    type Output = Item;

    fn index(&self, position: HexCoord) -> &Item {
        &self.grid[self.transform.apply(position) + self.offset]
    }
}

impl<'a, Layout: GridLayout, Item: Copy + Eq + Hash> Hash for Tile<'a, Layout, Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in self.iter() {
            value.hash(state);
        }
    }
}

impl<'a, Layout: GridLayout, Item: Copy + PartialEq + Eq + Hash> PartialEq
    for Tile<'a, Layout, Item>
{
    fn eq(&self, other: &Self) -> bool {
        self.layout.size() == other.layout.size()
            && self.iter().zip(other.iter()).all(|(s, o)| s.eq(&o))
    }
}

impl<'a, Layout: GridLayout, Item: Copy + Eq + Hash> Eq for Tile<'a, Layout, Item> {}

/// Construct tiles by iterating over the grid and applying the transformations.
pub fn extract_tiles<'a, Item: Copy + Eq + Hash>(
    grid: &'a Grid<HexagonalGridLayout, Item>,
    transforms: &'a [TransformMatrix],
) -> HashSet<Tile<'a, HexagonalGridLayout, Item>> {
    let inner_layout = HexagonalGridLayout {
        radius: grid.layout.radius - 1,
    };
    let layout = HexagonalGridLayout { radius: 2 };
    inner_layout
        .iter()
        .flat_map(|offset| {
            transforms.iter().map(move |transform| Tile {
                grid,
                offset,
                layout,
                transform,
            })
        })
        .collect()
}

pub fn standard_tile_transforms() -> Vec<TransformMatrix> {
    vec![
        Transform::Identity.into(),
        Transform::RotateClockwise60.into(),
        Transform::RotateClockwise120.into(),
        Transform::RotateClockwise180.into(),
        Transform::RotateClockwise240.into(),
        Transform::RotateClockwise300.into(),
        Transform::ReflectS.into(),
        TransformMatrix::from_transforms(
            [Transform::ReflectS, Transform::RotateClockwise60].into_iter(),
        ),
        TransformMatrix::from_transforms(
            [Transform::ReflectS, Transform::RotateClockwise120].into_iter(),
        ),
        TransformMatrix::from_transforms(
            [Transform::ReflectS, Transform::RotateClockwise180].into_iter(),
        ),
        TransformMatrix::from_transforms(
            [Transform::ReflectS, Transform::RotateClockwise240].into_iter(),
        ),
        TransformMatrix::from_transforms(
            [Transform::ReflectS, Transform::RotateClockwise300].into_iter(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::{extract_tiles, Tile};
    use crate::hexgrid::layout::HexagonalGridLayout;
    use crate::hexgrid::{Grid, GridLayout, HexCoord, Transform, TransformMatrix};
    use crate::zone::Terrain;

    fn standard_transforms() -> Vec<TransformMatrix> {
        vec![
            Transform::Identity.into(),
            Transform::RotateClockwise60.into(),
            Transform::RotateClockwise120.into(),
            Transform::RotateClockwise180.into(),
            Transform::RotateClockwise240.into(),
            Transform::RotateClockwise300.into(),
            Transform::ReflectS.into(),
            TransformMatrix::from_transforms(
                [Transform::ReflectS, Transform::RotateClockwise60].into_iter(),
            ),
            TransformMatrix::from_transforms(
                [Transform::ReflectS, Transform::RotateClockwise120].into_iter(),
            ),
            TransformMatrix::from_transforms(
                [Transform::ReflectS, Transform::RotateClockwise180].into_iter(),
            ),
            TransformMatrix::from_transforms(
                [Transform::ReflectS, Transform::RotateClockwise240].into_iter(),
            ),
            TransformMatrix::from_transforms(
                [Transform::ReflectS, Transform::RotateClockwise300].into_iter(),
            ),
        ]
    }

    fn sample_map() -> Grid<HexagonalGridLayout, Terrain> {
        let layout = HexagonalGridLayout { radius: 3 };
        Grid {
            layout,
            data: vec![
                Terrain::Ocean,
                Terrain::Ocean,
                Terrain::Ocean,
                Terrain::Ocean,
                Terrain::Ocean,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Ocean,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Forest,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Forest,
                Terrain::Forest,
                Terrain::Forest,
                Terrain::Forest,
                Terrain::Forest,
            ],
        }
    }

    #[test]
    fn iter_origin() {
        let storage = &sample_map();
        let tile = Tile {
            grid: storage,
            offset: HexCoord::new(0, 0),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::Identity.into(),
        };
        let items: Vec<_> = tile.iter().collect();
        assert_eq!(
            items,
            vec![
                Terrain::Ocean,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Forest
            ]
        );
    }

    #[test]
    fn iter_offset() {
        let storage = &sample_map();
        let tile = Tile {
            grid: storage,
            offset: HexCoord::new(1, 0),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::Identity.into(),
        };
        let items: Vec<_> = tile.iter().collect();
        assert_eq!(
            items,
            vec![
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Mountain,
                Terrain::Forest,
                Terrain::Forest,
                Terrain::Forest
            ]
        );
    }

    #[test]
    fn equal() {
        let grid = &sample_map();
        let tile_a = Tile {
            grid,
            offset: HexCoord::new(1, 0),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::Identity.into(),
        };
        let tile_b = Tile {
            grid,
            offset: HexCoord::new(-1, 1),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::Identity.into(),
        };
        let tile_c = Tile {
            grid,
            offset: HexCoord::new(0, 0),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::Identity.into(),
        };
        assert_eq!(tile_a, tile_b);
        assert_ne!(tile_a, tile_c);
        assert_ne!(tile_b, tile_c);
    }

    #[test]
    fn compatible() {
        let grid = &sample_map();
        let tile_a = Tile {
            grid,
            offset: HexCoord::new(1, -1),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::Identity.into(),
        };
        let tile_b = Tile {
            grid,
            offset: HexCoord::new(-1, 1),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::Identity.into(),
        };
        assert!(tile_a.compatible_with(&tile_b, HexCoord::new(0, 1)));
        assert!(!tile_a.compatible_with(&tile_b, HexCoord::new(1, 0)));
        assert!(tile_b.compatible_with(&tile_a, HexCoord::new(0, -1)));
    }

    #[test]
    fn extract() {
        let storage = &sample_map();
        let transforms = &standard_transforms();
        let tiles = extract_tiles(storage, transforms);
        assert_eq!(tiles.len(), 30);
    }

    #[test]
    fn transform() {
        let grid = &sample_map();
        let tile = Tile {
            grid,
            offset: (0, -1).into(),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::RotateClockwise300.into(),
        };
        for (coord, data) in tile.layout.iter().zip(tile.iter()) {
            println!("{:?} {:?}", coord, data);
            assert_eq!(data, tile[coord]);
        }
    }

    #[test]
    fn compatible_transform() {
        let grid = &sample_map();
        let tile = Tile {
            grid,
            offset: (0, -1).into(),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::Identity.into(),
        };
        let tilex = Tile {
            grid,
            offset: (0, -1).into(),
            layout: HexagonalGridLayout { radius: 2 },
            transform: &Transform::RotateClockwise300.into(),
        };
        assert!(tile.compatible_with(&tilex, (-1, 0).into()));
    }
}
