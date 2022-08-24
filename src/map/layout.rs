use crate::map::HexCoord;

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

#[cfg(test)]
mod tests {
    use crate::{HexCoord, MapLayout};

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
