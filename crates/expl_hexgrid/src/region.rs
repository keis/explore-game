use super::{Grid, GridLayout, HexCoord};
use disjoint_hash_set::DisjointHashSet;

#[derive(Debug)]
pub struct Region(pub Vec<HexCoord>);

impl FromIterator<HexCoord> for Region {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = HexCoord>,
    {
        Region(Vec::<HexCoord>::from_iter(iter))
    }
}

pub struct Regions<'a, Layout, Item, Predicate>
where
    Layout: GridLayout,
    Predicate: Fn(&Item, &Item) -> bool,
{
    grid: &'a Grid<Layout, Item>,
    predicate: Predicate,
}

impl<'a, Layout, Item, Predicate> Regions<'a, Layout, Item, Predicate>
where
    Layout: GridLayout,
    Predicate: Fn(&Item, &Item) -> bool,
{
    pub fn from(grid: &'a Grid<Layout, Item>, predicate: Predicate) -> Self {
        Self { grid, predicate }
    }

    /// Find contiguous regions of the grid by the given binary predicate.
    pub fn iter(self) -> impl Iterator<Item = Region> {
        self.grid
            .iter()
            .map(|(coord, item)| {
                // Find a compatible neighbour and join with that region or form a single element
                // region
                if let Some(neighbour) = coord.neighbours().find(|&neighbour| {
                    self.grid
                        .get(neighbour)
                        .is_some_and(|other| (self.predicate)(item, other))
                }) {
                    (coord, neighbour)
                } else {
                    (coord, coord)
                }
            })
            .collect::<DisjointHashSet<_>>()
            .sets()
            .map(|mut coords| coords.drain().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::Regions;
    use crate::{layout::SquareGridLayout, Grid, GridLayout};

    #[test]
    fn regions() {
        let layout = SquareGridLayout {
            width: 3,
            height: 3,
        };
        let grid = Grid::with_data(
            layout,
            vec![
                0, 0, 1, //
                0, 0, 2, //
                1, 1, 1,
            ],
        );

        let regions: Vec<_> = Regions::from(&grid, |a, b| a == b).iter().collect();
        println!("regions {:?}", regions);
        assert_eq!(regions.len(), 4);
        assert_eq!(
            regions.iter().map(|r| r.0.len()).sum::<usize>(),
            layout.size()
        );
    }
}
