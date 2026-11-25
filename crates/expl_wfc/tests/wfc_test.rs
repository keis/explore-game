use expl_hexgrid::{layout::HexagonalGridLayout, Grid, GridLayout, HexCoord, Transform};
use expl_wfc::{
    cell::Cell,
    generator::DEFAULT_BUDGET_FACTOR,
    seed::Seed,
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, LoadGrid},
    Generator, Template, WFCError,
};
use more_asserts::assert_le;
use serde::{Deserialize, Serialize};
use serde_jsonlines::{json_lines, write_json_lines};
use std::{collections::BTreeMap, fs::File, io};

fn sample_map() -> Result<Grid<HexagonalGridLayout, char>, &'static str> {
    let mut file =
        io::BufReader::new(File::open("res/test.txt").map_err(|_| "failed to open file")?);
    Grid::<HexagonalGridLayout, char>::load(&mut file).map_err(|_| "infallible")
}

fn sample_template() -> Template<char> {
    let input = sample_map().unwrap();
    let wrapped_input = wrap_grid(input);
    let transforms = standard_tile_transforms();
    Template::from_tiles(extract_tiles(&wrapped_input, &transforms))
}

fn verify_generator_invariants(generator: &Generator<HexagonalGridLayout, char>) {
    for coord in generator.grid.layout.iter() {
        match generator.grid[coord] {
            Cell::Collapsed(_) => {
                assert_ne!(
                    generator.collapsed.iter().find(|(cc, _, _)| *cc == coord),
                    None,
                    "expected to find {:?} in collapsed history",
                    coord,
                );
            }
            Cell::Alternatives(num_alts, _) => {
                if num_alts < generator.template.available_tiles() {
                    assert_ne!(
                        generator.pending.iter().find(|(qc, _)| **qc == coord),
                        None,
                        "expected to find {:?} in queue",
                        coord,
                    );
                }
            }
        };
    }
}

/// Every pair of neighbouring collapsed cells must be compatible per the template.
fn verify_neighbour_compatibility(generator: &Generator<HexagonalGridLayout, char>) {
    for coord in generator.grid.layout.iter() {
        let Some(Cell::Collapsed(tile)) = generator.grid.get(coord) else {
            continue;
        };
        for (offset, compatible) in generator.template.compatible_tiles(*tile) {
            if let Some(Cell::Collapsed(other)) = generator.grid.get(coord + offset) {
                assert!(
                    compatible.contains(*other),
                    "tile {} at {:?} is not compatible with tile {} at {:?}",
                    tile,
                    coord,
                    other,
                    coord + offset,
                );
            }
        }
    }
}

/// The D13 log measure: one `(|rejected|, committed)` element per history depth, plus the
/// pending retry (`rejected` of the cell in `next`) as an uncommitted element.
type LogMeasure = Vec<(usize, bool)>;

fn log_measure(generator: &Generator<HexagonalGridLayout, char>) -> LogMeasure {
    let mut measure: LogMeasure = generator
        .collapsed
        .iter()
        .map(|(_, _, rejected)| (rejected.len(), true))
        .collect();
    if let Some(rejected) = &generator.rejected {
        if !rejected.is_empty() {
            measure.push((rejected.len(), false));
        }
    }
    measure
}

/// Asserts the log measure is strictly greater, lexicographically, than the previous one.
/// `false` (uncommitted) orders before `true` (committed) at equal size and an extension is
/// greater than its prefix, which is what `Vec`'s `Ord` gives.
#[derive(Default)]
struct ProgressCheck {
    previous: Option<LogMeasure>,
}

impl ProgressCheck {
    fn check(&mut self, generator: &Generator<HexagonalGridLayout, char>) {
        self.record(log_measure(generator));
    }

    fn record(&mut self, measure: LogMeasure) {
        if let Some(previous) = &self.previous {
            assert!(
                measure > *previous,
                "log measure did not increase: {:?} -> {:?}",
                previous,
                measure,
            );
        }
        self.previous = Some(measure);
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
struct GeneratorTraceStep {
    pub coord: HexCoord,
    pub tile: usize,
    pub pending: BTreeMap<HexCoord, usize>,
}

#[test]
fn test_collapse() {
    let template = sample_template();
    println!("{:?}", template.stats());

    let mut generator = Generator::new_with_layout(&template, HexagonalGridLayout { radius: 5 });
    println!("map size is {:?}", generator.grid.layout.size());
    let mut steps = 0;
    let mut progress = ProgressCheck::default();
    while generator.step().unwrap().is_some() {
        verify_generator_invariants(&generator);
        verify_neighbour_compatibility(&generator);
        progress.check(&generator);
        steps += 1;
        assert_le!(steps, 1000);
    }
    let output = generator.export().unwrap();

    assert_eq!(output.layout.radius, 5);
}

#[test]
fn test_fixed_seed() {
    let template = sample_template();
    println!("{:?}", template.stats());

    let seed: Seed = "AAFP26SGQFDAYVCFVE".parse().unwrap();
    let mut generator: Generator<HexagonalGridLayout, char> =
        Generator::new_with_seed(&template, seed).unwrap();
    println!("map size is {:?}", generator.grid.layout.size());

    let expected_trace = json_lines("res/trace.jsonl")
        .and_then(|data| data.collect::<Result<Vec<GeneratorTraceStep>, _>>())
        .unwrap();
    let mut new_trace = vec![];
    let mut expected_trace_iter = expected_trace.iter();

    let mut steps = 0;
    let mut progress = ProgressCheck::default();
    while generator.step().unwrap().is_some() {
        verify_generator_invariants(&generator);
        verify_neighbour_compatibility(&generator);
        progress.check(&generator);
        steps += 1;
        let last = generator.collapsed.last().unwrap();
        let step = GeneratorTraceStep {
            coord: last.0,
            tile: last.1,
            pending: generator.pending.clone().drain().collect(),
        };
        if let Some(expected) = expected_trace_iter.next() {
            assert_eq!(step, *expected);
        }
        new_trace.push(step);
    }
    let output = generator.export().unwrap();

    assert_eq!(steps, 325);
    let stats = generator.stats();
    assert_eq!(stats.collapses, 298);
    assert_eq!(stats.rewinds, 27);
    assert_eq!(stats.steps, stats.collapses + stats.rewinds);
    assert_eq!(output.layout.radius, 10);

    write_json_lines("res/trace.jsonl", &new_trace).unwrap();
}

#[test]
fn test_fixed_seed_statistics() {
    let template = sample_template();
    let seed: Seed = "AAFP26SGQFDAYVCFVE".parse().unwrap();
    let mut generator: Generator<HexagonalGridLayout, char> =
        Generator::new_with_seed(&template, seed).unwrap();
    generator.record_events(false);
    while generator.step().unwrap().is_some() {}

    let stats = *generator.stats();
    assert_eq!(stats.steps, 325);
    assert_eq!(stats.collapses, 298);
    assert_eq!(stats.rewinds, 27);
    assert_eq!(stats.max_backtrack_depth, 5);
    assert_eq!(stats.stale_failures, 0);

    let events = generator.events();
    assert_eq!(events.len(), stats.collapses);
    assert!(events.iter().all(|event| event.pending.is_none()));
    assert_eq!(events.iter().filter(|event| !event.completed).count(), 15);
}

#[test]
fn test_default_budget_is_factor_of_cell_count() {
    let template = sample_template();
    let generator: Generator<HexagonalGridLayout, char> =
        Generator::new_with_layout(&template, HexagonalGridLayout { radius: 5 });
    assert_eq!(
        generator.budget(),
        DEFAULT_BUDGET_FACTOR * generator.grid.layout.size()
    );
}

fn run_with_budget(budget: usize) -> (Result<(), WFCError>, usize) {
    let template = sample_template();
    let seed: Seed = "AAFP26SGQFDAYVCFVE".parse().unwrap();
    let mut generator: Generator<HexagonalGridLayout, char> =
        Generator::new_with_seed(&template, seed)
            .unwrap()
            .with_budget(budget);
    let result = loop {
        match generator.step() {
            Ok(Some(())) => {}
            Ok(None) => break Ok(()),
            Err(error) => break Err(error),
        }
    };
    (result, generator.stats().collapses)
}

#[test]
fn test_tiny_budget_fails_at_same_collapse_count() {
    let first = run_with_budget(50);
    let second = run_with_budget(50);
    assert!(matches!(first.0, Err(WFCError::BudgetExceeded(50))));
    assert!(matches!(second.0, Err(WFCError::BudgetExceeded(50))));
    assert_eq!(first.1, 50);
    assert_eq!(second.1, 50);
}

#[test]
fn test_explicit_budget_overrides_default() {
    // The golden seed needs 298 collapses on 271 cells, below the default of
    // DEFAULT_BUDGET_FACTOR x 271, but above this explicit budget.
    assert!(matches!(
        run_with_budget(100).0,
        Err(WFCError::BudgetExceeded(100))
    ));
    // A budget above the default lets the same seed finish too.
    let (result, collapses) = run_with_budget(1_000_000);
    assert!(result.is_ok());
    assert_eq!(collapses, 298);
}

#[test]
fn test_unsatisfiable_template_returns_history_error() {
    // Every cell of the sample is unique, so the only tiles are the ones in the
    // sample itself and nothing can continue past the edge of it.
    let layout = HexagonalGridLayout { radius: 3 };
    let grid = Grid::<HexagonalGridLayout, char>::with_data(
        layout,
        ('a'..).take(layout.size()).collect::<Vec<_>>(),
    );
    let transforms = vec![Transform::Identity.into()];
    let template = Template::from_tiles(extract_tiles(&grid, &transforms));
    assert_eq!(template.available_tiles(), 7);

    let mut generator: Generator<HexagonalGridLayout, char> =
        Generator::new_with_layout(&template, HexagonalGridLayout { radius: 3 });
    let result = loop {
        match generator.step() {
            Ok(Some(())) => {}
            Ok(None) => break Ok(()),
            Err(error) => break Err(error),
        }
    };
    assert!(matches!(result, Err(WFCError::HistoryExhausted)));
}

#[test]
fn test_progress_check_accepts_increasing_measures() {
    let mut progress = ProgressCheck::default();
    progress.record(vec![(0, true)]);
    // extension
    progress.record(vec![(0, true), (0, true)]);
    // rewind: top element grows and is uncommitted
    progress.record(vec![(0, true), (1, false)]);
    // retry collapse commits it
    progress.record(vec![(0, true), (1, true)]);
    // further rewind pops down to a shallower element
    progress.record(vec![(1, false)]);
}

#[test]
#[should_panic(expected = "log measure did not increase")]
fn test_progress_check_rejects_repeated_measure() {
    let mut progress = ProgressCheck::default();
    progress.record(vec![(0, true), (1, false)]);
    progress.record(vec![(0, true), (1, false)]);
}

#[test]
#[should_panic(expected = "log measure did not increase")]
fn test_progress_check_rejects_decreasing_measure() {
    let mut progress = ProgressCheck::default();
    progress.record(vec![(0, true), (1, true)]);
    // a rewind that only drops the top entry without growing anything
    progress.record(vec![(0, true)]);
}
