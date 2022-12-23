use super::hexcoord::HexCoord;
use super::layout::MapLayout;

pub struct MapStorage<T> {
    pub layout: MapLayout,
    pub data: Vec<T>,
}

impl<T> MapStorage<T> {
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
