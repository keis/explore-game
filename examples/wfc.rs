use clap::{Args, Parser, Subcommand};
use explore_game::{
    hexgrid::layout::{HexagonalGridLayout, SquareGridLayout},
    hexgrid::{Grid, GridLayout},
    map::Terrain,
    wfc::{
        cell::Cell,
        tile::{extract_tiles, standard_tile_transforms},
        util::{wrap_grid, DumpGrid, DumpGridWith, LoadGrid},
        Generator, Seed, SeedType, Template,
    },
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
    let wrapped_input = wrap_grid(input);
    let transforms = standard_tile_transforms();
    Template::from_tiles(extract_tiles(&wrapped_input, &transforms))
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(global = true, long)]
    seed: Option<Seed>,
    #[arg(global = true, long)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    Hexagonal(HexagonalArgs),
    Square(SquareArgs),
}

#[derive(Args, Debug)]
struct HexagonalArgs {
    radius: u16,
}

#[derive(Args, Debug)]
struct SquareArgs {
    width: u16,
    height: u16,
}

fn generate_map<Layout>(
    template: &Template<Terrain>,
    generator: &mut Generator<Layout, Terrain>,
    verbose: bool,
) where
    Layout: GridLayout,
    Grid<Layout, Terrain>: DumpGrid,
    Grid<Layout, Cell>: DumpGridWith<Item = Cell>,
{
    while generator.step().is_some() {
        if verbose {
            generator
                .grid
                .dump_with(&mut io::stdout(), |cell| match cell {
                    Cell::Collapsed(tile) => template.contribution(*tile).into(),
                    Cell::Alternatives(alts, _) if alts < &template.available_tiles() => '?',
                    Cell::Alternatives(_, _) => '.',
                })
                .unwrap();
        }
    }
    let output = generator.export().unwrap();
    output.dump(&mut io::stdout()).unwrap();
}

fn main() -> Result<(), &'static str> {
    let args = Cli::parse();

    let template = sample_template();
    println!("{:?}", template.stats());

    match args.command {
        Command::Hexagonal(params) => {
            let seed = args
                .seed
                .unwrap_or_else(|| Seed::new(SeedType::Hexagonal(params.radius)));
            println!("Generating map with seed {}", seed);
            let mut generator: Generator<HexagonalGridLayout, Terrain> =
                Generator::new_with_seed(&template, seed)?;
            generate_map(&template, &mut generator, args.verbose);
            Ok(())
        }
        Command::Square(params) => {
            let seed = args
                .seed
                .unwrap_or_else(|| Seed::new(SeedType::Square(params.width, params.height)));
            println!("Generating map with seed {}", seed);
            let mut generator: Generator<SquareGridLayout, Terrain> =
                Generator::new_with_seed(&template, seed)?;
            generate_map(&template, &mut generator, args.verbose);
            Ok(())
        }
    }
}
