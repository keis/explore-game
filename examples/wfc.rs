use explore_game::{
    hexgrid::layout::HexagonalGridLayout,
    hexgrid::{Grid, GridLayout},
    wfc::cell::Cell,
    wfc::generator::Generator,
    wfc::template::Template,
    wfc::tile::{extract_tiles, standard_tile_transforms},
    wfc::util::{dump_grid, dump_grid_with, load_grid, wrap_grid},
    zone::Terrain,
};
use std::fs::File;
use std::io;

fn dump_output(template: &Template<Terrain>, grid: &Grid<HexagonalGridLayout, Cell>) {
    dump_grid_with(grid, &mut io::stdout(), |cell| match cell {
        Cell::Collapsed(tile) => template.contribution(*tile).into(),
        Cell::Alternatives(_, _) => '?',
    })
    .unwrap();
}

fn sample_map() -> Result<Grid<HexagonalGridLayout, Terrain>, &'static str> {
    let mut file =
        io::BufReader::new(File::open("assets/maps/test.txt").map_err(|_| "failed to open file")?);
    load_grid(&mut file)
}

fn sample_template() -> Template<Terrain> {
    let input = sample_map().unwrap();
    dump_grid(&input, &mut io::stdout()).unwrap();
    let wrapped_input = wrap_grid(input);
    let transforms = standard_tile_transforms();
    Template::from_tiles(extract_tiles(&wrapped_input, &transforms))
}

fn main() {
    let mut rng = rand::thread_rng();
    let template = sample_template();
    println!("{:?}", template.stats());

    let mut generator = Generator::new(&template, HexagonalGridLayout { radius: 5 });
    println!("map size is {:?}", generator.grid.layout.size());
    while generator.step(&mut rng).is_some() {
        dump_output(generator.template, &generator.grid);
    }
    let output = generator.export().unwrap();
    dump_grid(&output, &mut io::stdout()).unwrap();
}
