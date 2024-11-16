use clap::{Args, Parser, Subcommand};
use expl_hexgrid::{
    layout::{HexagonalGridLayout, SquareGridLayout},
    Grid, GridLayout,
};
use expl_wfc::{
    cell::Cell,
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, DumpGrid, DumpGridWith, LoadGrid},
    Generator, Seed, SeedType, Template, WFCError,
};
use std::fs::File;
use std::io;

fn sample_grid() -> Result<Grid<HexagonalGridLayout, char>, &'static str> {
    let mut file =
        io::BufReader::new(File::open("res/test.txt").map_err(|_| "failed to open file")?);
    Grid::<HexagonalGridLayout, char>::load(&mut file).map_err(|_| "infallible")
}

fn sample_template() -> Template<char> {
    let input = sample_grid().unwrap();
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
    template: &Template<char>,
    generator: &mut Generator<Layout, char>,
    verbose: bool,
) where
    Layout: GridLayout,
    Grid<Layout, char>: DumpGrid,
    Grid<Layout, Cell>: DumpGridWith<Item = Cell>,
{
    while generator.step().is_some() {
        if verbose {
            generator
                .grid
                .dump_with(&mut io::stdout(), |cell| match cell {
                    Cell::Collapsed(tile) => template.contribution(*tile),
                    Cell::Alternatives(alts, _) if alts < &template.available_tiles() => '?',
                    Cell::Alternatives(_, _) => '.',
                })
                .unwrap();
        }
    }
    let output = generator.export().unwrap();
    output.dump(&mut io::stdout()).unwrap();
}

fn main() -> Result<(), WFCError> {
    let args = Cli::parse();

    let template = sample_template();
    println!("{:?}", template.stats());

    match args.command {
        Command::Hexagonal(params) => {
            let seed = args
                .seed
                .unwrap_or_else(|| Seed::new(SeedType::Hexagonal(params.radius)));
            println!("Generating map with seed {}", seed);
            let mut generator: Generator<HexagonalGridLayout, char> =
                Generator::new_with_seed(&template, seed)?;
            generate_map(&template, &mut generator, args.verbose);
            Ok(())
        }
        Command::Square(params) => {
            let seed = args
                .seed
                .unwrap_or_else(|| Seed::new(SeedType::Square(params.width, params.height)));
            println!("Generating map with seed {}", seed);
            let mut generator: Generator<SquareGridLayout, char> =
                Generator::new_with_seed(&template, seed)?;
            generate_map(&template, &mut generator, args.verbose);
            Ok(())
        }
    }
}
