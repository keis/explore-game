use super::{HexCoord, Transform};

pub trait GridLayout: Copy + Clone + PartialEq {
    type LayoutIter<'a>: Iterator<Item = HexCoord> + ExactSizeIterator
    where
        Self: 'a;

    fn size(self) -> usize;
    fn iter(&'_ self) -> Self::LayoutIter<'_>;
    fn offset(&self, position: HexCoord) -> Option<usize>;
    fn contains(&self, position: HexCoord) -> bool;
    fn wrap(&self, position: HexCoord) -> HexCoord;
    fn center(&self) -> HexCoord;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SquareGridLayout {
    pub width: i32,
    pub height: i32,
}

impl GridLayout for SquareGridLayout {
    type LayoutIter<'a> = SquareGridLayoutIterator<'a>;

    fn size(self) -> usize {
        (self.width * self.height).try_into().unwrap()
    }

    fn iter(&'_ self) -> Self::LayoutIter<'_> {
        SquareGridLayoutIterator { layout: self, i: 0 }
    }

    fn offset(&self, position: HexCoord) -> Option<usize> {
        if !self.contains(position) {
            return None;
        }
        usize::try_from(position.r * self.width + position.q + position.r / 2)
            .ok()
            .filter(|o| o < &self.size())
    }

    fn contains(&self, position: HexCoord) -> bool {
        let qoffset = position.q + position.r / 2;
        position.r >= 0 && position.r < self.height && qoffset >= 0 && qoffset < self.width
    }

    fn wrap(&self, _position: HexCoord) -> HexCoord {
        panic!("Not implemented");
    }

    fn center(&self) -> HexCoord {
        HexCoord {
            q: self.width / 2 - self.height / 4,
            r: self.height / 2,
        }
    }
}

pub struct SquareGridLayoutIterator<'a> {
    layout: &'a SquareGridLayout,
    i: i32,
}

impl<'a> Iterator for SquareGridLayoutIterator<'a> {
    type Item = HexCoord;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remains = self.layout.size() - self.i as usize;
        (remains, Some(remains))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.layout.size() as i32 {
            return None;
        }

        let r = self.i / self.layout.width;
        let q = self.i % self.layout.width - r / 2;

        self.i += 1;
        Some(HexCoord::new(q, r))
    }
}

impl ExactSizeIterator for SquareGridLayoutIterator<'_> {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HexagonalGridLayout {
    pub radius: i32,
}

impl GridLayout for HexagonalGridLayout {
    type LayoutIter<'a> = HexagonalGridLayoutIterator<'a>;

    fn size(self) -> usize {
        (((self.radius - 1) * self.radius) * 3 + 1)
            .try_into()
            .unwrap()
    }

    fn iter(&'_ self) -> Self::LayoutIter<'_> {
        HexagonalGridLayoutIterator { layout: self, i: 0 }
    }

    fn offset(&self, position: HexCoord) -> Option<usize> {
        if !self.contains(position) {
            return None;
        }
        let rhombus_size = (self.radius - 1) * self.radius;
        let offset = if position.q >= 0 && position.r < 0 {
            position.q * (self.radius - 1) - position.r
        } else if position.r >= 0 && position.s() < 0 {
            rhombus_size + position.r * (self.radius - 1) - position.s()
        } else if position.s() >= 0 && position.q < 0 {
            rhombus_size * 2 + position.s() * (self.radius - 1) - position.q
        } else {
            0
        };
        usize::try_from(offset).ok().filter(|o| o < &self.size())
    }

    fn contains(&self, position: HexCoord) -> bool {
        position.distance(HexCoord::ZERO) <= (self.radius - 1) as u32
    }

    fn wrap(&self, position: HexCoord) -> HexCoord {
        let base = HexCoord::new(2 * self.radius - 1, 1 - self.radius);
        let mirror_center = [
            Transform::Identity,
            Transform::RotateClockwise60,
            Transform::RotateClockwise120,
            Transform::RotateClockwise180,
            Transform::RotateClockwise240,
            Transform::RotateClockwise300,
        ]
        .iter()
        .map(|transform| transform.apply(base))
        .min_by_key(|mc| position.distance(*mc))
        .unwrap();
        let mut result = position - mirror_center;
        while !self.contains(result) {
            result -= mirror_center;
        }
        result
    }

    fn center(&self) -> HexCoord {
        HexCoord::ZERO
    }
}

pub struct HexagonalGridLayoutIterator<'a> {
    layout: &'a HexagonalGridLayout,
    i: i32,
}

impl<'a> Iterator for HexagonalGridLayoutIterator<'a> {
    type Item = HexCoord;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remains = self.layout.size() - self.i as usize;
        (remains, Some(remains))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let rhombus_height = self.layout.radius - 1;
        let rhombus_size = rhombus_height * self.layout.radius;
        let result = if self.i == 0 {
            Some(HexCoord::ZERO)
        } else if self.i <= rhombus_size {
            let offset = self.i - 1;
            Some(HexCoord::new(
                offset / rhombus_height,
                -(offset % rhombus_height) - 1,
            ))
        } else if self.i <= rhombus_size * 2 {
            let offset = self.i - rhombus_size - 1;
            Some(HexCoord::new_rs(
                offset / rhombus_height,
                -(offset % rhombus_height) - 1,
            ))
        } else if self.i <= rhombus_size * 3 {
            let offset = self.i - rhombus_size * 2 - 1;
            Some(HexCoord::new_qs(
                -(offset % rhombus_height) - 1,
                offset / rhombus_height,
            ))
        } else {
            None
        };
        self.i += 1;
        result
    }
}

impl ExactSizeIterator for HexagonalGridLayoutIterator<'_> {}

#[cfg(test)]
mod tests {
    use super::{GridLayout, HexCoord, HexagonalGridLayout, SquareGridLayout};

    #[test]
    fn square_map_3x3() {
        let layout = SquareGridLayout {
            width: 3,
            height: 3,
        };
        assert_eq!(layout.iter().len(), 9);
        let coords: Vec<HexCoord> = layout.iter().collect();

        println!("coords {:?}", coords);
        assert_eq!(coords.len(), 9);
        assert_eq!(coords[0], HexCoord::ZERO);
        assert_eq!(layout.offset(HexCoord::ZERO), Some(0));
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

        assert!(layout.contains(HexCoord::ZERO));
        assert!(layout.contains(HexCoord::new(-1, 2)));
        assert!(!layout.contains(HexCoord::new(-2, 2)));
    }

    #[test]
    fn square_offset_matches_iter() {
        let layout = SquareGridLayout {
            width: 8,
            height: 8,
        };
        let offsets: Vec<_> = layout.iter().map(|coord| layout.offset(coord)).collect();
        assert_eq!(offsets, (0..layout.size()).map(Some).collect::<Vec<_>>());
    }

    #[test]
    fn hexagonal_map_r2() {
        let layout = HexagonalGridLayout { radius: 2 };
        let coords: Vec<HexCoord> = layout.iter().collect();

        assert_eq!(layout.iter().len(), 7);
        println!("coords {:?}", coords);
        assert_eq!(layout.size(), 7);
        assert_eq!(coords.len(), 7);
        assert_eq!(coords[0], HexCoord::ZERO);
        assert_eq!(coords[1], HexCoord::new(0, -1));
        assert_eq!(coords[2], HexCoord::new(1, -1));
        assert_eq!(coords[5], HexCoord::new(-1, 1));
        assert_eq!(layout.offset(HexCoord::ZERO), Some(0));
        assert_eq!(layout.offset(HexCoord::new(0, -1)), Some(1));
        assert_eq!(layout.offset(HexCoord::new(-1, 1)), Some(5));

        assert!(layout.contains(HexCoord::ZERO));
        assert!(layout.contains(HexCoord::new(-1, 1)));
        assert!(!layout.contains(HexCoord::new(-2, 1)));
    }

    #[test]
    fn hexagonal_map_iter_r3() {
        let layout = HexagonalGridLayout { radius: 3 };
        let coords: Vec<HexCoord> = layout.iter().collect();

        println!("coords {:?}", coords);
        assert_eq!(layout.size(), 19);
        assert_eq!(coords.len(), 19);
        assert_eq!(coords[0], HexCoord::ZERO);
        assert_eq!(coords[1], HexCoord::new(0, -1));
        assert_eq!(coords[2], HexCoord::new(0, -2));
        assert_eq!(coords[3], HexCoord::new(1, -1));
        assert_eq!(coords[7], HexCoord::new(1, 0));
        assert_eq!(coords[13], HexCoord::new(-1, 1));
        assert_eq!(layout.offset(HexCoord::new(-2, 0)), Some(18));
    }

    #[test]
    fn hexagonal_offset_matches_iter() {
        let layout = HexagonalGridLayout { radius: 5 };
        let coords: Vec<HexCoord> = layout.iter().collect();
        println!("coords {:?}", coords);
        let offsets: Vec<_> = layout.iter().map(|coord| layout.offset(coord)).collect();
        assert_eq!(offsets, (0..layout.size()).map(Some).collect::<Vec<_>>());
    }

    #[test]
    fn test_wrap_coord() {
        let layout = HexagonalGridLayout { radius: 3 };
        assert_eq!(layout.wrap(HexCoord::new(3, 0)), HexCoord::new(-2, 2));
    }
}
