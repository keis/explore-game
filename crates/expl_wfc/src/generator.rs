use super::{
    cell::Cell,
    seed::{Seed, SeedType},
    template::Template,
    TileId, WFCError,
};
use expl_hexgrid::{Grid, GridLayout, HexCoord};
use fixedbitset::FixedBitSet;
use rand::Rng;
use std::{
    cmp::Reverse,
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

/// Run statistics for a generator, readable by the caller.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Statistics {
    /// Calls to `step` that did work.
    pub steps: usize,
    /// Collapse events (cells collapsed, including ones later undone).
    pub collapses: usize,
    /// History entries popped by `rewind`.
    pub rewinds: usize,
    /// Most collapses undone by one uninterrupted sequence of rewinds.
    pub max_backtrack_depth: usize,
    /// Failures to select on a cell that is neither the cell just restored by
    /// rewind nor a neighbour of the last collapse.
    pub stale_failures: usize,
}

/// A collapse as it happened, logged by the opt-in recorder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollapseEvent {
    pub coord: HexCoord,
    pub tile: TileId,
    /// Propagation finished without leaving a neighbour with no alternatives.
    pub completed: bool,
    /// Pending cells after propagation, only recorded when `completed` and the
    /// recorder was asked for snapshots.
    pub pending: Option<BTreeMap<HexCoord, usize>>,
}

struct Recorder {
    with_pending: bool,
    events: Vec<CollapseEvent>,
}

/// Generator is the state of the iterative process for generating a map using WFC
pub struct Generator<'a, Layout: GridLayout, Item> {
    pub template: &'a Template<Item>,
    pub grid: Grid<Layout, Cell>,
    pub collapsed: Vec<(HexCoord, TileId, Vec<TileId>)>, // Coordinate, selected tile, rejected tiles
    pub pending: HashMap<HexCoord, usize>,
    pub rejected: Option<Vec<TileId>>,
    next: Option<HexCoord>,
    stats: Statistics,
    backtrack_run: usize,
    last_restored: Option<HexCoord>,
    last_collapsed: Option<HexCoord>,
    recorder: Option<Recorder>,
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
        Self::from_layout(template, layout, Seed::new(layout.into()))
    }

    pub fn new_with_seed(template: &'a Template<Item>, seed: Seed) -> Result<Self, WFCError>
    where
        Layout: TryFrom<SeedType> + TryFrom<SeedType>,
        WFCError: From<<Layout as TryFrom<SeedType>>::Error>,
    {
        let layout: Layout = seed.seed_type.try_into()?;
        Ok(Self::from_layout(template, layout, seed))
    }

    fn from_layout(template: &'a Template<Item>, layout: Layout, seed: Seed) -> Self {
        let default_cell = Cell::empty(template.available_tiles());
        let grid = Grid::with_fill(layout, default_cell);
        let collapsed: Vec<(HexCoord, TileId, Vec<TileId>)> =
            Vec::with_capacity(grid.layout.size());
        Self {
            template,
            grid,
            collapsed,
            pending: HashMap::new(),
            next: Some(layout.center()),
            rejected: Some(Vec::new()),
            stats: Statistics::default(),
            backtrack_run: 0,
            last_restored: None,
            last_collapsed: None,
            recorder: None,
            rand: seed.into(),
        }
    }

    pub fn stats(&self) -> &Statistics {
        &self.stats
    }

    /// Start logging every collapse as it happens (off by default).
    pub fn record_events(&mut self, with_pending: bool) {
        self.recorder = Some(Recorder {
            with_pending,
            events: Vec::new(),
        });
    }

    pub fn events(&self) -> &[CollapseEvent] {
        self.recorder.as_ref().map_or(&[], |r| &r.events)
    }

    pub fn rand(self) -> impl Rng {
        self.rand
    }

    pub fn alternatives(&self, coord: HexCoord) -> FixedBitSet {
        let mut alts = FixedBitSet::with_capacity(self.template.available_tiles());
        alts.set_range(.., true);
        for neighbour in coord.neighbours() {
            let Some(Cell::Collapsed(tile)) = self.grid.get(neighbour) else {
                continue;
            };
            for (offset, compatible) in self.template.compatible_tiles(*tile) {
                if neighbour + offset == coord {
                    alts.intersect_with(compatible);
                }
            }
        }
        alts
    }

    pub fn propagate(&mut self, coord: HexCoord, tile: TileId) {
        for (offset, compatible) in self.template.compatible_tiles(tile) {
            let neighbour = coord + offset;
            if let Some(neighbour_cell) = self.grid.get_mut(neighbour) {
                neighbour_cell.retain(compatible);
                if let Cell::Alternatives(num_alts, _) = neighbour_cell {
                    self.pending.insert(neighbour, *num_alts);
                }
            }
        }
    }

    pub fn rewind(&mut self) {
        let (last_coord, last_tile, mut last_rejected) = self.collapsed.pop().unwrap();
        self.stats.rewinds += 1;
        self.backtrack_run += 1;
        self.stats.max_backtrack_depth = self.stats.max_backtrack_depth.max(self.backtrack_run);
        self.last_restored = Some(last_coord);
        assert!(!last_rejected.contains(&last_tile));
        last_rejected.push(last_tile);
        let mut alternatives = self.alternatives(last_coord);
        for tile in &last_rejected {
            alternatives.set(*tile, false);
        }
        self.pending.insert(last_coord, alternatives.count_ones(..));
        self.grid[last_coord] = Cell::empty(self.template.available_tiles());
        self.grid[last_coord].set_alternatives(alternatives);
        for neighbour in last_coord.neighbours() {
            if let Some(Cell::Alternatives(_, _)) = self.grid.get(neighbour) {
                let alternatives = self.alternatives(neighbour);
                self.pending.insert(neighbour, alternatives.count_ones(..));
                self.grid[neighbour].set_alternatives(alternatives);
            }
        }
        // Because self.rejected refers to the cell at `last_coord` it is important this is the
        // next cell that gets collapsed.
        self.rejected = Some(last_rejected);
        self.next = Some(last_coord);
    }

    pub fn step(&mut self) -> Option<()> {
        let coord = self.next?;
        if let Some(tile) = self.grid[coord].select(&mut self.rand) {
            self.stats.steps += 1;
            self.stats.collapses += 1;
            self.backtrack_run = 0;
            self.grid[coord].collapse(tile);
            let rejected = self.rejected.replace(Vec::new()).unwrap();
            assert!(tile <= self.template.available_tiles());
            assert!(!rejected.contains(&tile));
            self.collapsed.push((coord, tile, rejected));
            self.propagate(coord, tile);
            self.pending.remove(&coord);
            self.last_collapsed = Some(coord);
            if self.recorder.is_some() {
                self.record_collapse(coord, tile);
            }
            self.next = self
                .pending
                .iter()
                .max_by_key(|(coord, score)| (Reverse(*score), coord.q, coord.r))
                .map(|(coord, _)| *coord);
        } else {
            self.stats.steps += 1;
            let expected = self.last_restored == Some(coord)
                || self
                    .last_collapsed
                    .is_some_and(|last| last.neighbours().any(|n| n == coord));
            if !expected {
                self.stats.stale_failures += 1;
            }
            self.rewind();
        }
        Some(())
    }

    fn record_collapse(&mut self, coord: HexCoord, tile: TileId) {
        let completed = coord
            .neighbours()
            .all(|n| !matches!(self.grid.get(n), Some(Cell::Alternatives(0, _))));
        let Some(recorder) = self.recorder.as_mut() else {
            return;
        };
        let pending = (completed && recorder.with_pending)
            .then(|| self.pending.iter().map(|(c, n)| (*c, *n)).collect());
        recorder.events.push(CollapseEvent {
            coord,
            tile,
            completed,
            pending,
        });
    }

    pub fn export(&self) -> Result<Grid<Layout, Item>, WFCError> {
        let data: Vec<_> = self
            .grid
            .iter_data()
            .map(|cell| match cell {
                Cell::Collapsed(tile) => Ok(self.template.contribution(*tile)),
                Cell::Alternatives(_, _) => Err(WFCError::CellNotCollapsed),
            })
            .collect::<Result<_, _>>()?;

        Ok(Grid::with_data(self.grid.layout, data))
    }
}
