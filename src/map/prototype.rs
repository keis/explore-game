use crate::hexgrid::{Grid, GridLayout};
use crate::zone::Zone;
use rand::Rng;

pub struct MapPrototype;

impl MapPrototype {
    pub fn generate<L: GridLayout>(layout: L) -> Grid<L, Zone> {
        let mut rng = rand::thread_rng();
        Grid {
            layout,
            data: layout.iter().map(|_| Zone { terrain: rng.gen() }).collect(),
        }
    }
}
