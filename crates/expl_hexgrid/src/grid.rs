use super::{GridLayout, HexCoord};
use std::fmt;
use std::ops::{Index, IndexMut};

pub struct Grid<L: GridLayout, T> {
    pub layout: L,
    pub data: Vec<T>,
}

impl<L: GridLayout, T> Grid<L, T> {
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
}
