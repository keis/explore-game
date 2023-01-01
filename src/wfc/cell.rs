use crate::wfc::TileId;
use rand::Rng;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum Cell {
    Collapsed(TileId),
    Alternatives(usize, Vec<bool>),
}

impl Cell {
    pub fn collapse<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        if let Cell::Alternatives(num_alts, alts) = self {
            if *num_alts == 0usize {
                return;
            }
            let choice = rng.gen_range(0..*num_alts);
            if let Some(tile_id) = alts
                .iter()
                .enumerate()
                .filter(|(_, x)| **x)
                .nth(choice)
                .map(|(i, _)| i as TileId)
            {
                *self = Cell::Collapsed(tile_id);
            }
        }
    }

    pub fn set_alternatives(&mut self, alternatives: &HashSet<TileId>) {
        if let Cell::Alternatives(num_alts, alts) = self {
            for (idx, allowed) in alts.iter_mut().enumerate() {
                *allowed = alternatives.contains(&idx);
            }
            *num_alts = alternatives.len();
        }
    }

    pub fn retain(&mut self, compatible: &HashSet<TileId>) {
        if let Cell::Alternatives(num_alts, alts) = self {
            for (idx, allowed) in alts.iter_mut().enumerate() {
                *allowed = *allowed && compatible.contains(&idx);
            }
            *num_alts = alts.iter().filter(|allowed| **allowed).count();
        }
    }

    pub fn reject<Iter: Iterator<Item = TileId>>(&mut self, rejected: Iter) {
        if let Cell::Alternatives(num_alts, alts) = self {
            for reject in rejected {
                alts[reject] = false;
            }
            *num_alts = alts.iter().filter(|allowed| **allowed).count();
        }
    }
}
