use super::{
    cell::Cell,
    seed::{Seed, SeedType},
    template::Template,
    TileId,
};
use expl_hexgrid::{Grid, GridLayout, HexCoord};
use fixedbitset::FixedBitSet;
use rand::RngCore;
use std::cmp::Reverse;
use std::hash::Hash;

/// Generator is the state of the iterative process for generating a map using WFC
pub struct Generator<'a, Layout: GridLayout, Item> {
    pub template: &'a Template<Item>,
    pub grid: Grid<Layout, Cell>,
    pub collapsed: Vec<(HexCoord, TileId, Vec<TileId>)>, // Coordinate, selected tile, rejected tiles
    pub queue: Vec<HexCoord>,
    pub rejected: Option<Vec<TileId>>,
    rand: rand_xoshiro::Xoshiro256PlusPlus,
}

impl<'a, Layout, Item> Generator<'a, Layout, Item>
where
    Layout: GridLayout,
    Item: Copy + PartialEq + Ord + Hash,
{
    pub fn new_with_layout(template: &'a Template<Item>, layout: Layout) -> Self
    where
        Layout: Into<SeedType>,
    {
        let default_cell = Cell::empty(template.available_tiles());
        let seed = Seed::new(layout.into());
        let grid = Grid::with_fill(layout, default_cell);
        let collapsed: Vec<(HexCoord, TileId, Vec<TileId>)> =
            Vec::with_capacity(grid.layout.size());
        Self {
            template,
            grid,
            collapsed,
            queue: vec![layout.center()],
            rejected: Some(Vec::new()),
            rand: seed.into(),
        }
    }

    pub fn new_with_seed(template: &'a Template<Item>, seed: Seed) -> Result<Self, &'static str>
    where
        Layout: TryFrom<SeedType> + TryFrom<SeedType>,
        &'static str: From<<Layout as TryFrom<SeedType>>::Error>,
    {
        let default_cell = Cell::empty(template.available_tiles());
        let layout: Layout = seed.seed_type.try_into()?;
        let grid = Grid::with_fill(layout, default_cell);
        let collapsed: Vec<(HexCoord, TileId, Vec<TileId>)> =
            Vec::with_capacity(grid.layout.size());
        Ok(Self {
            template,
            grid,
            collapsed,
            queue: vec![layout.center()],
            rejected: Some(Vec::new()),
            rand: seed.into(),
        })
    }

    pub fn rand(self) -> impl RngCore {
        self.rand
    }

    pub fn alternatives(&self, coord: HexCoord) -> FixedBitSet {
        let mut alts = FixedBitSet::with_capacity(self.template.available_tiles());
        alts.set_range(.., true);
        for neighbour in coord.neighbours() {
            let Some(Cell::Collapsed(tile)) = self.grid.get(neighbour) else { continue };
            for (offset, compatible) in self.template.compatible_tiles(*tile) {
                if neighbour + *offset == coord {
                    alts.intersect_with(compatible);
                }
            }
        }
        alts
    }

    pub fn propagate(&mut self, coord: HexCoord, tile: TileId) {
        for (offset, compatible) in self.template.compatible_tiles(tile) {
            let neighbour = coord + *offset;
            if let Some(neighbour_cell) = self.grid.get_mut(neighbour) {
                neighbour_cell.retain(compatible);
                if let Cell::Alternatives(_, _) = neighbour_cell {
                    self.queue.push(neighbour);
                }
            }
        }
    }

    pub fn rewind(&mut self) {
        let (last_coord, last_tile, mut last_rejected) = self.collapsed.pop().unwrap();
        assert!(!last_rejected.contains(&last_tile));
        last_rejected.push(last_tile);
        let mut alternatives = self.alternatives(last_coord);
        for tile in &last_rejected {
            alternatives.set(*tile, false);
        }
        self.grid[last_coord] = Cell::empty(self.template.available_tiles());
        self.grid[last_coord].set_alternatives(alternatives);
        for neighbour in last_coord.neighbours() {
            if let Some(Cell::Alternatives(_, _)) = self.grid.get(neighbour) {
                let alternatives = self.alternatives(neighbour);
                self.grid[neighbour].set_alternatives(alternatives)
            }
        }
        // Because self.rejected refers to the cell at `last_coord` it is important this is the
        // next cell that gets collapsed. This could probably be done in some better way.
        self.rejected = Some(last_rejected);
        self.queue.push(last_coord);
    }

    pub fn step(&mut self) -> Option<()> {
        let coord = self.queue.pop()?;
        self.grid[coord].collapse(&mut self.rand);
        if let Cell::Collapsed(tile) = self.grid[coord] {
            let rejected = self.rejected.replace(Vec::new()).unwrap();
            assert!(tile <= self.template.available_tiles());
            assert!(!rejected.contains(&tile));
            self.collapsed.push((coord, tile, rejected));
            self.propagate(coord, tile);
            self.queue.sort_by_key(|e| match self.grid[*e] {
                Cell::Collapsed(tile) => {
                    panic!("Collapsed cell in queue {:?} {:?}", e, tile,);
                }
                Cell::Alternatives(num_alts, _) => (Reverse(num_alts), e.q, e.r),
            });
            self.queue.dedup();
        } else {
            self.queue.push(coord);
            self.rewind();
        }
        Some(())
    }

    pub fn export(&self) -> Result<Grid<Layout, Item>, &'static str> {
        let data: Vec<_> = self
            .grid
            .iter_data()
            .map(|cell| match cell {
                Cell::Collapsed(tile) => Ok(self.template.contribution(*tile)),
                Cell::Alternatives(_, _) => Err("Cell not collapsed"),
            })
            .collect::<Result<_, _>>()?;

        Ok(Grid::with_data(self.grid.layout, data))
    }
}
