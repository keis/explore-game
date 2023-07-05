use super::TileId;
use fixedbitset::FixedBitSet;
use rand::Rng;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cell {
    Collapsed(TileId),
    Alternatives(usize, FixedBitSet),
}

impl Cell {
    pub fn empty(num_alts: usize) -> Self {
        let mut alts = FixedBitSet::with_capacity(num_alts);
        alts.set_range(.., true);
        Self::Alternatives(num_alts, alts)
    }

    pub fn select<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<TileId> {
        if let Cell::Alternatives(num_alts, alts) = self {
            if *num_alts == 0usize {
                return None;
            }
            let choice = rng.gen_range(0..*num_alts);
            if let Some(tile_id) = alts.ones().nth(choice).map(|i| i as TileId) {
                return Some(tile_id);
            }
        }
        None
    }

    pub fn collapse(&mut self, tile_id: TileId) {
        *self = Cell::Collapsed(tile_id);
    }

    pub fn set_alternatives(&mut self, alternatives: FixedBitSet) {
        if let Cell::Alternatives(num_alts, alts) = self {
            *alts = alternatives;
            *num_alts = alts.count_ones(..);
        }
    }

    pub fn retain(&mut self, compatible: &FixedBitSet) {
        if let Cell::Alternatives(num_alts, alts) = self {
            alts.intersect_with(compatible);
            *num_alts = alts.count_ones(..);
        }
    }
}
