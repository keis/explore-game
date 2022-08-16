use crate::HexCoord;
use bevy::prelude::*;
use pathfinding::prelude::astar;

#[derive(Copy, Clone)]
pub struct MapLayout {
    pub width: isize,
    pub height: isize,
}

impl MapLayout {
    pub fn size(self) -> usize {
        (self.width * self.height).try_into().unwrap()
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = HexCoord> + '_ {
        MapLayoutIterator { layout: self, i: 0 }
    }

    pub fn offset(&self, position: HexCoord) -> Option<usize> {
        usize::try_from(position.r * self.width + position.q + position.r / 2)
            .ok()
            .filter(|o| o < &self.size())
    }
}

pub struct Map {
    tiles: Vec<Option<Entity>>,
    pub layout: MapLayout,
}

impl Map {
    pub fn new(layout: MapLayout) -> Self {
        Self {
            layout,
            tiles: vec![None; layout.size()],
        }
    }

    pub fn set(&mut self, position: HexCoord, entity: Option<Entity>) {
        if let Some(offset) = self.layout.offset(position) {
            self.tiles[offset] = entity;
        }
    }

    pub fn get(&self, position: HexCoord) -> Option<Entity> {
        self.layout
            .offset(position)
            .and_then(|offset| self.tiles[offset])
    }
}

pub struct MapLayoutIterator<'a> {
    layout: &'a MapLayout,
    i: isize,
}

impl<'a> Iterator for MapLayoutIterator<'a> {
    type Item = HexCoord;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.i / self.layout.width;
        let q = self.i % self.layout.width - r / 2;

        if r >= self.layout.height {
            return None;
        }

        self.i += 1;
        Some(HexCoord::new(q, r))
    }
}

#[derive(Component)]
pub struct MapComponent {
    pub map: Map,
}

pub fn find_path(
    start: HexCoord,
    goal: HexCoord,
    is_walkable: &impl Fn(&HexCoord) -> bool,
) -> Option<(Vec<HexCoord>, u32)> {
    astar(
        &start,
        |p| {
            p.neighbours()
                .into_iter()
                .filter(is_walkable)
                .map(|p| (p, 1))
                .collect::<Vec<(HexCoord, u32)>>()
        },
        |p| p.distance(&goal).try_into().unwrap(),
        |p| *p == goal,
    )
}

#[cfg(test)]
mod tests {
    use crate::{find_path, HexCoord, MapLayout};

    #[test]
    fn pathfinding_neighbour() {
        let start = HexCoord::new(2, 4);
        let goal = HexCoord::new(2, 3);

        let result = find_path(start, goal, &|_| true);
        println!("neigbours {:?}", start.neighbours());
        println!("path {:?}", result);
        assert_eq!(result.expect("no path found").1, 1);
    }

    #[test]
    fn pathfinding() {
        let start = HexCoord::new(0, 0);
        let goal = HexCoord::new(4, 2);

        let result = find_path(start, goal, &|_| true);
        println!("path {:?}", result);
        assert_eq!(result.expect("no path found").1, 6);
    }

    #[test]
    fn map_iter() {
        let layout = MapLayout {
            width: 3,
            height: 3,
        };
        let coords: Vec<HexCoord> = layout.iter().collect();

        println!("coords {:?}", coords);
        assert_eq!(coords.len(), 9);
        assert_eq!(coords[0], HexCoord::new(0, 0));
        assert_eq!(layout.offset(HexCoord::new(0, 0)).expect("in bounds"), 0);
        assert_eq!(coords[1], HexCoord::new(1, 0));
        assert_eq!(coords[2], HexCoord::new(2, 0));
        assert_eq!(coords[3], HexCoord::new(0, 1));
        assert_eq!(coords[4], HexCoord::new(1, 1));
        assert_eq!(coords[5], HexCoord::new(2, 1));
        assert_eq!(coords[6], HexCoord::new(-1, 2));
        assert_eq!(layout.offset(HexCoord::new(-1, 2)).expect("in bounds"), 6);
        assert_eq!(coords[7], HexCoord::new(0, 2));
        assert_eq!(coords[8], HexCoord::new(1, 2));
        assert_eq!(layout.offset(HexCoord::new(1, 2)).expect("in bounds"), 8);
    }
}
