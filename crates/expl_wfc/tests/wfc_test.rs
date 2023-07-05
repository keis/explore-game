use expl_hexgrid::{layout::HexagonalGridLayout, Grid, GridLayout, HexCoord};
use expl_wfc::{
    cell::Cell,
    seed::Seed,
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, LoadGrid},
    Generator, Template,
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
    while generator.step().is_some() {
        verify_generator_invariants(&generator);
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
    while generator.step().is_some() {
        verify_generator_invariants(&generator);
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

    assert_eq!(steps, 405);
    assert_eq!(output.layout.radius, 10);

    write_json_lines("res/trace.jsonl", &new_trace).unwrap();
}
