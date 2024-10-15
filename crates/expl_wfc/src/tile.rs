use expl_hexgrid::{
    layout::HexagonalGridLayout, Grid, GridLayout, HexCoord, Transform, TransformMatrix,
};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Index;

/// A Tile acts as a lense into a grid where a possibly rotated or reflected subsection can be accessed.
#[derive(Copy, Clone, Debug)]
pub struct Tile<'a, Layout: GridLayout, Item> {
    grid: &'a Grid<Layout, Item>,
    offset: HexCoord,
    // Note that transform is applied to coordinates so effectivly the tile is rotated in the *opposite*
    // direction
    transform: &'a TransformMatrix,
}

impl<Layout: GridLayout, Item: Copy + PartialEq> Tile<'_, Layout, Item> {
    const COORDS: [HexCoord; 7] = [
        HexCoord::new(0, -1),
        HexCoord::new(1, -1),
        HexCoord::new(-1, 0),
        HexCoord::new(0, 0),
        HexCoord::new(1, 0),
        HexCoord::new(-1, 1),
        HexCoord::new(0, 1),
    ];

    pub fn iter(&self) -> impl '_ + Iterator<Item = Item> {
        Self::COORDS.iter().map(|&coord| self[coord])
    }

    pub fn compatible_with(&self, other: &Tile<Layout, Item>, offset: HexCoord) -> bool {
        Self::COORDS
            .iter()
            .all(|&coord| (coord - offset).length() > 1 || self[coord] == other[coord - offset])
    }
}

impl<Layout: GridLayout, Item> Index<HexCoord> for Tile<'_, Layout, Item> {
    type Output = Item;

    fn index(&self, position: HexCoord) -> &Item {
        &self.grid[self.transform.apply(position) + self.offset]
    }
}

impl<Layout: GridLayout, Item: Copy + Eq + Hash> Hash for Tile<'_, Layout, Item> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in self.iter() {
            value.hash(state);
        }
    }
}

impl<Layout: GridLayout, Item: Copy + PartialEq + Eq + Hash> PartialEq for Tile<'_, Layout, Item> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().zip(other.iter()).all(|(s, o)| s.eq(&o))
    }
}

impl<Layout: GridLayout, Item: Copy + Eq + Hash> Eq for Tile<'_, Layout, Item> {}

impl<Layout: GridLayout, Item: Copy + PartialEq + Eq + Hash + Ord + PartialOrd> Ord
    for Tile<'_, Layout, Item>
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<Layout: GridLayout, Item: Copy + PartialEq + Eq + Hash + Ord + PartialOrd> PartialOrd
    for Tile<'_, Layout, Item>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Construct tiles by iterating over the grid and applying the transformations.
pub fn extract_tiles<'a, Item: Copy + Eq + Hash>(
    grid: &'a Grid<HexagonalGridLayout, Item>,
    transforms: &'a [TransformMatrix],
) -> HashSet<Tile<'a, HexagonalGridLayout, Item>> {
    let inner_layout = HexagonalGridLayout {
        radius: grid.layout.radius - 1,
    };
    inner_layout
        .iter()
        .flat_map(|offset| {
            transforms.iter().map(move |transform| Tile {
                grid,
                offset,
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
pub mod tests {
    use super::{extract_tiles, standard_tile_transforms, Tile};
    use crate::util::LoadGrid;
    use expl_hexgrid::{layout::HexagonalGridLayout, Grid, HexCoord, Transform, TransformMatrix};
    use rstest::*;
    use std::cmp::Ordering;
    use std::{fs::File, io};

    #[fixture]
    pub fn standard_transforms() -> Vec<TransformMatrix> {
        standard_tile_transforms()
    }

    #[fixture]
    pub fn sample_map() -> Grid<HexagonalGridLayout, char> {
        let mut file = io::BufReader::new(File::open("res/test-r3.txt").unwrap());
        Grid::<HexagonalGridLayout, char>::load(&mut file).unwrap()
    }

    #[rstest]
    fn iter_origin(sample_map: Grid<HexagonalGridLayout, char>) {
        let tile = Tile {
            grid: &sample_map,
            offset: HexCoord::new(0, 0),
            transform: &Transform::Identity.into(),
        };
        let items: Vec<_> = tile.iter().collect();
        assert_eq!(items, vec!['~', '^', '^', '^', '^', '^', '%']);
    }

    #[rstest]
    fn iter_offset(sample_map: Grid<HexagonalGridLayout, char>) {
        let tile = Tile {
            grid: &sample_map,
            offset: HexCoord::new(1, 0),
            transform: &Transform::Identity.into(),
        };
        let items: Vec<_> = tile.iter().collect();
        assert_eq!(items, vec!['^', '^', '^', '^', '%', '%', '%']);
    }

    #[rstest]
    fn equal(sample_map: Grid<HexagonalGridLayout, char>) {
        let tile_a = Tile {
            grid: &sample_map,
            offset: HexCoord::new(1, 0),
            transform: &Transform::Identity.into(),
        };
        let tile_b = Tile {
            grid: &sample_map,
            offset: HexCoord::new(-1, 1),
            transform: &Transform::Identity.into(),
        };
        let tile_c = Tile {
            grid: &sample_map,
            offset: HexCoord::new(0, 0),
            transform: &Transform::Identity.into(),
        };
        assert_eq!(tile_a, tile_b);
        assert_ne!(tile_a, tile_c);
        assert_ne!(tile_b, tile_c);
    }

    #[rstest]
    fn ordering(sample_map: Grid<HexagonalGridLayout, char>) {
        let tile_a = Tile {
            grid: &sample_map,
            offset: HexCoord::new(1, 0),
            transform: &Transform::Identity.into(),
        };
        let tile_b = Tile {
            grid: &sample_map,
            offset: HexCoord::new(-1, 1),
            transform: &Transform::Identity.into(),
        };
        let tile_c = Tile {
            grid: &sample_map,
            offset: HexCoord::new(0, 0),
            transform: &Transform::Identity.into(),
        };
        assert_eq!(tile_a.cmp(&tile_b), Ordering::Equal);
        assert_eq!(tile_a.cmp(&tile_c), Ordering::Less);
        assert_eq!(tile_b.cmp(&tile_c), Ordering::Less);
    }

    #[rstest]
    fn compatible(sample_map: Grid<HexagonalGridLayout, char>) {
        let tile_a = Tile {
            grid: &sample_map,
            offset: HexCoord::new(1, -1),
            transform: &Transform::Identity.into(),
        };
        let tile_b = Tile {
            grid: &sample_map,
            offset: HexCoord::new(-1, 1),
            transform: &Transform::Identity.into(),
        };
        assert!(tile_a.compatible_with(&tile_b, HexCoord::new(0, 1)));
        assert!(!tile_a.compatible_with(&tile_b, HexCoord::new(1, 0)));
        assert!(tile_b.compatible_with(&tile_a, HexCoord::new(0, -1)));
    }

    #[rstest]
    fn extract(
        standard_transforms: Vec<TransformMatrix>,
        sample_map: Grid<HexagonalGridLayout, char>,
    ) {
        let tiles = extract_tiles(&sample_map, &standard_transforms);
        assert_eq!(tiles.len(), 30);
    }

    #[rstest]
    fn transform(sample_map: Grid<HexagonalGridLayout, char>) {
        let tile = Tile {
            grid: &sample_map,
            offset: (0, -1).into(),
            transform: &Transform::RotateClockwise300.into(),
        };
        for (&coord, data) in Tile::<HexagonalGridLayout, char>::COORDS
            .iter()
            .zip(tile.iter())
        {
            println!("{:?} {:?}", coord, data);
            assert_eq!(data, tile[coord]);
        }
    }

    #[rstest]
    fn compatible_transform(sample_map: Grid<HexagonalGridLayout, char>) {
        let tile = Tile {
            grid: &sample_map,
            offset: (0, -1).into(),
            transform: &Transform::Identity.into(),
        };
        let tilex = Tile {
            grid: &sample_map,
            offset: (0, -1).into(),
            transform: &Transform::RotateClockwise300.into(),
        };
        assert!(tile.compatible_with(&tilex, (-1, 0).into()));
    }
}
