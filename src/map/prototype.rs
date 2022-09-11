use super::{HexCoord, MapLayout};
use crate::zone::Zone;
use rand::Rng;

pub struct MapPrototype {
    pub layout: MapLayout,
    zones: Vec<Zone>,
}

impl MapPrototype {
    pub fn generate(layout: MapLayout) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            layout,
            zones: layout.iter().map(|_| Zone { terrain: rng.gen() }).collect(),
        }
    }

    pub fn get(&self, position: HexCoord) -> Zone {
        let offset = self.layout.offset(position).unwrap();
        self.zones[offset]
    }
}
