use explore_game::{
    hexgrid::layout::HexagonalGridLayout,
    hexgrid::{ring, Grid, GridLayout, HexCoord},
    wfc::cell::Cell,
    wfc::generator::Generator,
    wfc::template::Template,
    wfc::tile::{extract_tiles, standard_tile_transforms, Tile},
    wfc::util::{dump_grid, dump_grid_with, load_grid},
    zone::Terrain,
};
use std::fs::File;
use std::io;

fn sample_map() -> Result<Grid<HexagonalGridLayout, Terrain>, &'static str> {
    let mut file =
        io::BufReader::new(File::open("assets/maps/test.txt").map_err(|_| "failed to open file")?);
    load_grid(&mut file)
}

fn dump_output(template: &Template<Terrain>, grid: &Grid<HexagonalGridLayout, Cell>) {
    dump_grid_with(grid, &mut io::stdout(), |cell| match cell {
        Cell::Collapsed(tile) => template.contribution(*tile).into(),
        Cell::Alternatives(_, _) => '?',
    });
}

fn dump_tile(tile: Tile<HexagonalGridLayout, Terrain>) {
    let mut lastr = 0;
    for coord in tile.layout.iter() {
        if coord.r != lastr {
            print!("\n{}", " ".repeat(coord.r.abs().try_into().unwrap()));
            lastr = coord.r;
        }
        print!(" {}", <Terrain as Into<char>>::into(tile[coord]));
    }
    println!();
}

fn wrap_grid(grid: Grid<HexagonalGridLayout, Terrain>) -> Grid<HexagonalGridLayout, Terrain> {
    let layout = HexagonalGridLayout {
        radius: grid.layout.radius + 1,
    };
    let mut wrapped = Grid {
        layout,
        data: vec![Terrain::Ocean; layout.size()],
    };
    for coord in grid.layout.iter() {
        wrapped[coord] = grid[coord];
    }
    for coord in ring(HexCoord::ZERO, grid.layout.radius + 1) {
        wrapped[coord] = grid[grid.layout.wrap(coord)];
    }
    wrapped
}

fn sample_template() -> Template<Terrain> {
    let input = sample_map().unwrap();
    dump_grid(&input, &mut io::stdout()).unwrap();
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
    dump_grid(&output, &mut io::stdout()).unwrap();

    assert_eq!(output.layout.radius, 5);
}
