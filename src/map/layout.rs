use crate::map::HexCoord;

pub trait MapLayout: Copy + Clone {
    type LayoutIter<'a>: Iterator<Item = HexCoord>
    where
        Self: 'a;

    fn size(self) -> usize;
    fn iter(&'_ self) -> Self::LayoutIter<'_>;
    fn offset(&self, position: HexCoord) -> Option<usize>;
}

#[derive(Copy, Clone)]
pub struct SquareMapLayout {
    pub width: isize,
    pub height: isize,
}

impl MapLayout for SquareMapLayout {
    type LayoutIter<'a> = SquareMapLayoutIterator<'a>;

    fn size(self) -> usize {
        (self.width * self.height).try_into().unwrap()
    }

    fn iter(&'_ self) -> Self::LayoutIter<'_> {
        SquareMapLayoutIterator { layout: self, i: 0 }
    }

    fn offset(&self, position: HexCoord) -> Option<usize> {
        usize::try_from(position.r * self.width + position.q + position.r / 2)
            .ok()
            .filter(|o| o < &self.size())
    }
}

pub struct SquareMapLayoutIterator<'a> {
    layout: &'a SquareMapLayout,
    i: isize,
}

impl<'a> Iterator for SquareMapLayoutIterator<'a> {
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

#[derive(Copy, Clone)]
pub struct HexagonalMapLayout {
    pub radius: usize,
}

impl MapLayout for HexagonalMapLayout {
    type LayoutIter<'a> = HexagonalMapLayoutIterator<'a>;

    fn size(self) -> usize {
        ((self.radius - 1) * self.radius) * 3 + 1
    }

    fn iter(&'_ self) -> Self::LayoutIter<'_> {
        HexagonalMapLayoutIterator {
            layout: self,
            q: 0,
            r: 1 - self.radius as isize,
        }
    }

    fn offset(&self, position: HexCoord) -> Option<usize> {
        let radius = self.radius as isize;
        let row = position.r + radius - 1;
        let qadjust = if position.r >= 0 {
            (radius - 1) * radius - (radius - position.r - 1) * (radius - position.r) / 2
        } else {
            row * (row + 1) / 2
        };
        // adjust for lowest q and increasing width (in neg these are related)
        usize::try_from(row * radius + qadjust + position.q)
            .ok()
            .filter(|o| o < &self.size())
    }
}

pub struct HexagonalMapLayoutIterator<'a> {
    layout: &'a HexagonalMapLayout,
    q: isize,
    r: isize,
}

impl<'a> Iterator for HexagonalMapLayoutIterator<'a> {
    type Item = HexCoord;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.r;
        let q = self.q;
        if self.r > self.layout.radius as isize - 1 {
            return None;
        }
        let qend = if r <= 0 {
            self.layout.radius as isize - 1
        } else {
            self.layout.radius as isize - r - 1
        };
        self.q += 1;
        if self.q > qend {
            self.r += 1;
            self.q = if self.r >= 0 {
                1 - self.layout.radius as isize
            } else {
                1 - self.r - self.layout.radius as isize
            }
        }
        Some(HexCoord::new(q, r))
    }
}

#[cfg(test)]
mod tests {
    use crate::map::{HexCoord, HexagonalMapLayout, MapLayout, SquareMapLayout};

    #[test]
    fn square_map_3x3() {
        let layout = SquareMapLayout {
            width: 3,
            height: 3,
        };
        let coords: Vec<HexCoord> = layout.iter().collect();

        println!("coords {:?}", coords);
        assert_eq!(coords.len(), 9);
        assert_eq!(coords[0], HexCoord::new(0, 0));
        assert_eq!(layout.offset(HexCoord::new(0, 0)), Some(0));
        assert_eq!(coords[1], HexCoord::new(1, 0));
        assert_eq!(coords[2], HexCoord::new(2, 0));
        assert_eq!(coords[3], HexCoord::new(0, 1));
        assert_eq!(coords[4], HexCoord::new(1, 1));
        assert_eq!(coords[5], HexCoord::new(2, 1));
        assert_eq!(coords[6], HexCoord::new(-1, 2));
        assert_eq!(layout.offset(HexCoord::new(-1, 2)), Some(6));
        assert_eq!(coords[7], HexCoord::new(0, 2));
        assert_eq!(coords[8], HexCoord::new(1, 2));
        assert_eq!(layout.offset(HexCoord::new(1, 2)), Some(8));
    }

    #[test]
    fn square_offset_matches_iter() {
        let layout = SquareMapLayout {
            width: 8,
            height: 8,
        };
        let offsets: Vec<_> = layout.iter().map(|coord| layout.offset(coord)).collect();
        assert_eq!(offsets, (0..layout.size()).map(Some).collect::<Vec<_>>());
    }

    #[test]
    fn hexagonal_map_r2() {
        let layout = HexagonalMapLayout { radius: 2 };
        let coords: Vec<HexCoord> = layout.iter().collect();

        println!("coords {:?}", coords);
        assert_eq!(layout.size(), 7);
        assert_eq!(coords.len(), 7);
        assert_eq!(coords[3], HexCoord::new(0, 0));
        assert_eq!(layout.offset(HexCoord::new(0, 0)), Some(3));
        assert_eq!(layout.offset(HexCoord::new(0, 1)), Some(6));
    }

    #[test]
    fn hexagonal_map_iter_r3() {
        let layout = HexagonalMapLayout { radius: 3 };
        let coords: Vec<HexCoord> = layout.iter().collect();

        println!("coords {:?}", coords);
        assert_eq!(layout.size(), 19);
        assert_eq!(coords.len(), 19);
        assert_eq!(coords[7], HexCoord::new(-2, 0));
        assert_eq!(layout.offset(HexCoord::new(-2, 0)), Some(7));
    }

    #[test]
    fn hexagonal_offset_matches_iter() {
        let layout = HexagonalMapLayout { radius: 5 };
        let coords: Vec<HexCoord> = layout.iter().collect();
        println!("coords {:?}", coords);
        let offsets: Vec<_> = layout.iter().map(|coord| layout.offset(coord)).collect();
        assert_eq!(offsets, (0..layout.size()).map(Some).collect::<Vec<_>>());
    }
}
