use explore_game::{
    hexgrid::layout::{HexagonalGridLayout, SquareGridLayout},
    hexgrid::{Grid, GridLayout},
    map::Terrain,
    wfc::cell::Cell,
    wfc::generator::Generator,
    wfc::template::Template,
    wfc::tile::{extract_tiles, standard_tile_transforms},
    wfc::util::{wrap_grid, DumpGrid, DumpGridWith, LoadGrid},
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

fn main() {
    let mut rng = rand::thread_rng();
    let template = sample_template();
    println!("{:?}", template.stats());

    let mut generator = Generator::new(
        &template,
        SquareGridLayout {
            width: 10,
            height: 8,
        },
    );
    println!("map size is {:?}", generator.grid.layout.size());
    while generator.step(&mut rng).is_some() {
        generator
            .grid
            .dump_with(&mut io::stdout(), |cell| match cell {
                Cell::Collapsed(tile) => template.contribution(*tile).into(),
                Cell::Alternatives(_, _) => '?',
            })
            .unwrap();
    }
    let output = generator.export().unwrap();
    output.dump(&mut io::stdout()).unwrap();
}
