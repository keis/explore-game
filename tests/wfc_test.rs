use explore_game::{
    hexgrid::layout::HexagonalGridLayout,
    hexgrid::{Grid, GridLayout},
    map::Terrain,
    wfc::cell::Cell,
    wfc::generator::Generator,
    wfc::template::Template,
    wfc::tile::{extract_tiles, standard_tile_transforms},
    wfc::util::{wrap_grid, DumpGrid, LoadGrid},
};
use std::fs::File;
use std::io;

fn sample_map() -> Result<Grid<HexagonalGridLayout, Terrain>, &'static str> {
    let mut file =
        io::BufReader::new(File::open("assets/maps/test.txt").map_err(|_| "failed to open file")?);
    Grid::<HexagonalGridLayout, Terrain>::load(&mut file)
}

fn sample_template() -> Template<Terrain> {
    let input = sample_map().unwrap();
    input.dump(&mut io::stdout()).unwrap();
    let wrapped_input = wrap_grid(input);
    let transforms = standard_tile_transforms();
    Template::from_tiles(extract_tiles(&wrapped_input, &transforms))
}

#[test]
fn test_collapse() {
    let mut rng = rand::thread_rng();
    let template = sample_template();
    println!("{:?}", template.stats());

    let mut generator = Generator::new(&template, HexagonalGridLayout { radius: 5 });
    println!("map size is {:?}", generator.grid.layout.size());
    while generator.step(&mut rng).is_some() {
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
    output.dump(&mut io::stdout()).unwrap();

    assert_eq!(output.layout.radius, 5);
}