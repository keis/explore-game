use bevy::prelude::*;
use expl_hexgrid::HexCoord;
use smallvec::SmallVec;

#[derive(Component)]
pub struct Combat {
    pub(super) position: HexCoord,
    pub(super) initiative_order: SmallVec<[Entity; 8]>,
    pub(super) initiative: usize,
}
