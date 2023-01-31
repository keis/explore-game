use crate::hexgrid::HexCoord;
use std::iter;

/// Iterate over the coordinates forming a ring of the given radius around `center`.
pub fn ring(center: HexCoord, radius: i32) -> impl Iterator<Item = HexCoord> {
    let mut coord = center + HexCoord::new(0, -1) * (radius - 1);
    HexCoord::ZERO
        .neighbours()
        .flat_map(move |n| iter::repeat(n).take((radius - 1) as usize))
        .map(move |step| {
            coord += step;
            coord
        })
}

/// Iterate over the coordinates forming rings around `center` with incrementally larger radius.
///
/// Note: This is an unbounded iterator.
/// Note: The coordinates when moving "rings" are not adjacent.
pub fn spiral(center: HexCoord) -> impl Iterator<Item = HexCoord> {
    iter::once(center).chain((1..).flat_map(move |r| ring(center, r)))
}

#[cfg(test)]
mod tests {
    use super::{ring, spiral, HexCoord};
    use itertools::Itertools;

    #[test]
    fn test_ring() {
        let coords: Vec<_> = ring(HexCoord::ZERO, 2).collect();
        assert_eq!(coords.len(), 6);
        for coord in coords {
            assert_eq!(coord.distance(HexCoord::ZERO), 1);
        }
    }

    #[test]
    fn test_spiral() {
        let center = HexCoord::new(4, 4);
        let spiralgroups = spiral(center).group_by(|coord| coord.distance(center));
        let mut sgiter = spiralgroups.into_iter();

        let (distance, group) = sgiter.next().unwrap();
        assert_eq!(distance, 0);
        assert_eq!(group.count(), 1);

        let (distance, group) = sgiter.next().unwrap();
        assert_eq!(distance, 1);
        assert_eq!(group.count(), 6);

        let (distance, group) = sgiter.next().unwrap();
        assert_eq!(distance, 2);
        assert_eq!(group.count(), 12);
    }
}
