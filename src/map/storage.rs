use super::hexcoord::HexCoord;
use super::layout::MapLayout;
use std::ops::{Index, IndexMut};

pub struct MapStorage<L: MapLayout, T> {
    pub layout: L,
    pub data: Vec<T>,
}

impl<L: MapLayout, T> MapStorage<L, T> {
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

impl<L: MapLayout, T> Index<HexCoord> for MapStorage<L, T> {
    type Output = T;

    fn index(&self, position: HexCoord) -> &T {
        &self.data[self.layout.offset(position).unwrap()]
    }
}

impl<L: MapLayout, T> IndexMut<HexCoord> for MapStorage<L, T> {
    fn index_mut(&mut self, position: HexCoord) -> &mut T {
        &mut self.data[self.layout.offset(position).unwrap()]
    }
}

#[cfg(test)]
mod tests {
    use crate::map::{HexCoord, MapLayout, MapStorage, SquareMapLayout};

    #[test]
    fn mutate_and_read_back() {
        let layout = SquareMapLayout {
            width: 3,
            height: 3,
        };
        let mut storage = MapStorage {
            layout,
            data: vec![0; layout.size()],
        };

        storage.set(HexCoord::new(1, 2), 17);
        assert_eq!(storage.get(HexCoord::new(1, 2)), Some(&17));
        assert_eq!(storage[HexCoord::new(1, 2)], 17);

        storage[HexCoord::new(2, 1)] = 13;
        assert_eq!(storage.get(HexCoord::new(2, 1)), Some(&13));
        assert_eq!(storage[HexCoord::new(2, 1)], 13);
    }
}
