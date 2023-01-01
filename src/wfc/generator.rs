use crate::hexgrid::{Grid, GridLayout, HexCoord};
use crate::wfc::{cell::Cell, template::Template, TileId};
use rand::Rng;
use std::cmp::Reverse;
use std::collections::HashSet;

/// Generator is the state of the iterative process for generating a map using WFC
pub struct Generator<'a, Layout: GridLayout, TileLayout: GridLayout, Item> {
    pub template: &'a Template<'a, TileLayout, Item>,
    pub grid: Grid<Layout, Cell>,
    pub collapsed: Vec<(HexCoord, TileId, Vec<TileId>)>, // Coordinate, selected tile, rejected tiles
    pub queue: Vec<HexCoord>,
    pub rejected: Option<Vec<TileId>>,
}

impl<'a, Layout: GridLayout, TileLayout: GridLayout, Item> Generator<'a, Layout, TileLayout, Item>
where
    Item: Copy + PartialEq,
{
    pub fn new(template: &'a Template<'a, TileLayout, Item>, layout: Layout) -> Self {
        let default_cell =
            Cell::Alternatives(template.tiles.len(), vec![true; template.tiles.len()]);
        let grid = Grid {
            layout,
            data: vec![default_cell; layout.size()],
        };
        let collapsed: Vec<(HexCoord, TileId, Vec<TileId>)> =
            Vec::with_capacity(grid.layout.size());
        Self {
            template,
            grid,
            collapsed,
            queue: vec![HexCoord::new(0, 0)],
            rejected: Some(Vec::new()),
        }
    }

    pub fn alternatives(&self, coord: HexCoord) -> HashSet<TileId> {
        let mut alts: HashSet<TileId> = (0..self.template.tiles.len())
            .map(|id| id as TileId)
            .collect();
        for neighbour in coord.neighbours() {
            if let Some(Cell::Collapsed(tile)) = self.grid.get(neighbour) {
                for (offset, compatible) in self.template.compatible_tiles(*tile) {
                    if neighbour + *offset == coord {
                        alts = alts.intersection(compatible).cloned().collect();
                    }
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
            alternatives.remove(tile);
        }
        self.grid[last_coord] = Cell::Alternatives(
            alternatives.len(),
            (0..self.template.tiles.len())
                .map(|id| alternatives.contains(&(id as TileId)))
                .collect(),
        );
        for neighbour in last_coord.neighbours() {
            if let Some(Cell::Alternatives(_, _)) = self.grid.get(neighbour) {
                let alternatives = self.alternatives(neighbour);
                self.grid[neighbour].set_alternatives(&alternatives)
            }
        }
        self.rejected = Some(last_rejected);
        self.queue.push(last_coord);
    }

    pub fn step<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<()> {
        let coord = self.queue.pop()?;
        self.grid[coord].collapse(rng);
        if let Cell::Collapsed(tile) = self.grid[coord] {
            assert!(tile <= self.template.tiles.len());
            self.collapsed
                .push((coord, tile, self.rejected.replace(Vec::new()).unwrap()));
            self.propagate(coord, tile);
        } else {
            self.rewind();
            self.queue.push(coord);
        }
        self.queue.sort_by_key(|e| match self.grid[*e] {
            Cell::Collapsed(tile) => {
                panic!("Collapsed cell in queue {:?} {:?}", e, tile,);
            }
            Cell::Alternatives(num_alts, _) => (Reverse(num_alts), e.q, e.r),
        });
        self.queue.dedup();
        Some(())
    }

    pub fn export(&self) -> Result<Grid<Layout, Item>, &'static str> {
        let data = self
            .grid
            .data
            .iter()
            .map(|cell| match cell {
                Cell::Collapsed(tile) => Ok(self.template.tiles[*tile][HexCoord::ZERO]),
                Cell::Alternatives(_, _) => Err("Cell not collapsed"),
            })
            .collect::<Result<_, _>>()?;

        Ok(Grid {
            layout: self.grid.layout,
            data,
        })
    }
}
