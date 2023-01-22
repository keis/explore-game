use crate::hexgrid::HexCoord;
use std::iter;

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

#[cfg(test)]
mod tests {
    use super::{ring, HexCoord};

    #[test]
    fn test_ring() {
        let coords: Vec<_> = ring(HexCoord::ZERO, 2).collect();
        assert_eq!(coords.len(), 6);
        for coord in coords {
            assert_eq!(coord.distance(&HexCoord::ZERO), 1);
        }
    }
}
