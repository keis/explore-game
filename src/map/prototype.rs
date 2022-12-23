use super::{MapLayout, MapStorage};
use crate::zone::Zone;
use rand::Rng;

pub struct MapPrototype;

impl MapPrototype {
    pub fn generate(layout: MapLayout) -> MapStorage<Zone> {
        let mut rng = rand::thread_rng();
        MapStorage {
            layout,
            data: layout.iter().map(|_| Zone { terrain: rng.gen() }).collect(),
        }
    }
}
