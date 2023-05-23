use expl_hexgrid::{layout::HexagonalGridLayout, Grid, GridLayout};
use expl_wfc::{
    cell::Cell,
    seed::Seed,
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, LoadGrid},
    Generator, Template,
};
use more_asserts::assert_le;
use std::{fs::File, io};

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
                    None
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

    let mut steps = 0;
    while generator.step().is_some() {
        verify_generator_invariants(&generator);
        steps += 1;
    }
    let output = generator.export().unwrap();

    assert_eq!(steps, 405);
    assert_eq!(output.layout.radius, 10);
}
