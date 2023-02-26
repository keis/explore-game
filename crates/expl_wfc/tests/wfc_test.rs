use expl_hexgrid::{layout::HexagonalGridLayout, Grid, GridLayout};
use expl_wfc::{
    cell::Cell,
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, LoadGrid},
    Generator, Template,
};
use std::fs::File;
use std::io;

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

#[test]
fn test_collapse() {
    let template = sample_template();
    println!("{:?}", template.stats());

    let mut generator = Generator::new_with_layout(&template, HexagonalGridLayout { radius: 5 });
    println!("map size is {:?}", generator.grid.layout.size());
    while generator.step().is_some() {
        for coord in generator.grid.layout.iter() {
            match generator.grid[coord] {
                Cell::Collapsed(_) => {
                    assert_ne!(
                        generator.collapsed.iter().find(|(cc, _, _)| *cc == coord),
                        None
                    );
                }
                Cell::Alternatives(num_alts, _) => {
                    if num_alts < template.available_tiles() {
                        assert_ne!(
                            generator.queue.iter().find(|qc| **qc == coord),
                            None,
                            "expected to find {:?} in queue",
                            coord,
                        );
                    }
                }
            };
        }
    }
    let output = generator.export().unwrap();

    assert_eq!(output.layout.radius, 5);
}
