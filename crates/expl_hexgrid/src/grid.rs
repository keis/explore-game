use crate::{GridLayout, HexCoord};
use std::fmt;
use std::ops::{Index, IndexMut};

pub struct Grid<L: GridLayout, T> {
    pub layout: L,
    data: Vec<T>,
}

impl<L: GridLayout, T> Grid<L, T> {
    /// Create a new grid of the given layout filled with the default value of T
    pub fn new(layout: L) -> Self
    where
        T: Default + Clone,
    {
        Self {
            layout,
            data: vec![T::default(); layout.size()],
        }
    }

    /// Create a new grid of the given layout filled with clones of the specified T
    pub fn with_fill(layout: L, fill: T) -> Self
    where
        T: Clone,
    {
        Self {
            layout,
            data: vec![fill; layout.size()],
        }
    }

    /// Create a new grid of the given layout and data in the internal iteration order.
    pub fn with_data<Iter>(layout: L, iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = T>,
    {
        let data: Vec<_> = iter.into_iter().collect();
        assert!(data.len() == layout.size());
        Self { layout, data }
    }

    /// An iterator visiting all coordinate-value pairs of the grid
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (HexCoord, &T)> {
        self.layout.iter().zip(self.data.iter())
    }

    /// An iterator visiting all values contained in the grid
    pub fn iter_data(&self) -> impl ExactSizeIterator<Item = &T> {
        self.data.iter()
    }

    pub fn set(&mut self, position: HexCoord, value: T) {
        if let Some(offset) = self.layout.offset(position) {
            self.data[offset] = value;
        }
    }

    pub fn get(&self, position: HexCoord) -> Option<&T> {
        self.layout
            .offset(position)
            .and_then(|offset| self.data.get(offset))
    }

    pub fn get_mut(&mut self, position: HexCoord) -> Option<&mut T> {
        self.layout
            .offset(position)
            .and_then(|offset| self.data.get_mut(offset))
    }
}

impl<L: GridLayout + fmt::Debug, T> fmt::Debug for Grid<L, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapStorage")
            .field("layout", &self.layout)
            .finish()
    }
}

impl<L: GridLayout, T> Index<HexCoord> for Grid<L, T> {
    type Output = T;

    fn index(&self, position: HexCoord) -> &T {
        &self.data[self.layout.offset(position).unwrap()]
    }
}

impl<L: GridLayout, T> IndexMut<HexCoord> for Grid<L, T> {
    fn index_mut(&mut self, position: HexCoord) -> &mut T {
        &mut self.data[self.layout.offset(position).unwrap()]
    }
}

impl<L: GridLayout, T> Extend<(HexCoord, T)> for Grid<L, T> {
    fn extend<Iter: IntoIterator<Item = (HexCoord, T)>>(&mut self, iter: Iter) {
        for (coord, elem) in iter {
            let Some(offset) = self.layout.offset(coord) else {
                continue;
            };
            self.data[offset] = elem;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Grid, GridLayout, HexCoord};
    use crate::layout::SquareGridLayout;

    #[test]
    fn mutate_and_read_back() {
        let layout = SquareGridLayout {
            width: 3,
            height: 3,
        };
        let mut grid = Grid {
            layout,
            data: vec![0; layout.size()],
        };

        grid.set(HexCoord::new(1, 2), 17);
        assert_eq!(grid.get(HexCoord::new(1, 2)), Some(&17));
        assert_eq!(grid[HexCoord::new(1, 2)], 17);

        grid[HexCoord::new(2, 1)] = 13;
        assert_eq!(grid.get(HexCoord::new(2, 1)), Some(&13));
        assert_eq!(grid[HexCoord::new(2, 1)], 13);
    }

    #[test]
    fn extend() {
        let layout = SquareGridLayout {
            width: 3,
            height: 3,
        };
        let mut grid = Grid::new(layout);
        grid.extend(vec![(HexCoord::new(1, 1), 6), (HexCoord::new(2, 2), 7)]);
        assert_eq!(grid[HexCoord::new(0, 0)], 0);
        assert_eq!(grid[HexCoord::new(1, 1)], 6);
        assert_eq!(grid.get(HexCoord::new(2, 2)), None);
    }
}
