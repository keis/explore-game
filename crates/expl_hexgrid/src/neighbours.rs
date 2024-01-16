use super::HexCoord;
use std::ops::{Index, IndexMut};

#[cfg(feature = "bevy-reflect")]
use bevy_reflect::Reflect;

/// A wrapper around a fixed sized array for holding per neighbour information
#[derive(Clone)]
#[cfg_attr(feature = "bevy-reflect", derive(Reflect))]
pub struct Neighbours<T>([T; 6]);

impl<T> Neighbours<T> {
    #[inline]
    pub const fn new(data: [T; 6]) -> Self {
        Self(data)
    }

    #[inline]
    pub fn from_fn<F: FnMut(HexCoord) -> T>(f: F) -> Self {
        Self(HexCoord::NEIGHBOUR_OFFSETS.map(f))
    }

    #[inline]
    pub fn from_fn_around<F: FnMut(HexCoord) -> T>(origin: HexCoord, mut f: F) -> Self {
        Neighbours::from_fn(|offset| f(origin + offset))
    }

    #[inline]
    pub fn values(&self) -> &[T; 6] {
        &self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = (HexCoord, &T)> {
        HexCoord::NEIGHBOUR_OFFSETS
            .iter()
            .copied()
            .zip(self.0.iter())
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T> Neighbours<T>
where
    T: Copy,
{
    #[inline]
    pub fn map<F: FnMut(T) -> U, U>(self, f: F) -> Neighbours<U> {
        Neighbours(self.0.map(f))
    }
}

impl<T> Copy for Neighbours<T> where T: Copy {}

impl<T> Default for Neighbours<T>
where
    T: Default,
{
    fn default() -> Self {
        Self(<[T; 6]>::default())
    }
}

impl<T> Index<usize> for Neighbours<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for Neighbours<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[cfg(test)]
mod tests {
    use super::Neighbours;
    use crate::HexCoord;

    #[test]
    fn test_from_fn() {
        let neighbours = Neighbours::from_fn(|coord| coord.distance(HexCoord::new(-2, 2)));
        assert_eq!(neighbours.values(), &[3, 2, 1, 2, 3, 3]);
    }

    #[test]
    fn test_from_fn_around() {
        let neighbours = Neighbours::from_fn_around(HexCoord::new(-2, 2), |coord| {
            coord.distance(HexCoord::new(-2, 2))
        });
        assert_eq!(neighbours.values(), &[1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn test_iter() {
        let neighbours = Neighbours::new([1, 1, 2, 3, 5, 8]);
        let mut iter = neighbours.iter();
        assert_eq!(iter.next(), Some((HexCoord::new(1, 0), &1)));
        assert_eq!(iter.next(), Some((HexCoord::new(0, 1), &1)));
        assert_eq!(iter.next(), Some((HexCoord::new(-1, 1), &2)));
        assert_eq!(iter.next(), Some((HexCoord::new(-1, 0), &3)));
        assert_eq!(iter.next(), Some((HexCoord::new(0, -1), &5)));
        assert_eq!(iter.next(), Some((HexCoord::new(1, -1), &8)));
        assert_eq!(iter.next(), None);
    }
}
