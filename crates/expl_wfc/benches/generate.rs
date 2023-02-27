use criterion::{black_box, criterion_group, criterion_main, Criterion};
use expl_hexgrid::{layout::HexagonalGridLayout, Grid};
use expl_wfc::{
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, LoadGrid},
    Generator, Seed, Template,
};
use std::fs::File;
use std::io;

fn sample_grid() -> Result<Grid<HexagonalGridLayout, char>, &'static str> {
    let mut file =
        io::BufReader::new(File::open("res/test.txt").map_err(|_| "failed to open file")?);
    Grid::<HexagonalGridLayout, char>::load(&mut file).map_err(|_| "infallible")
}

fn template_from_grid(input: &Grid<HexagonalGridLayout, char>) -> Template<char> {
    let transforms = standard_tile_transforms();
    Template::from_tiles(extract_tiles(input, &transforms))
}

fn generate(grid: &Grid<HexagonalGridLayout, char>, seed: Seed) -> Grid<HexagonalGridLayout, char> {
    let template = template_from_grid(grid);
    let mut generator = Generator::new_with_seed(&template, seed).unwrap();
    while generator.step().is_some() {}
    generator.export().unwrap()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let grid = sample_grid().unwrap();
    let wrapped_input = wrap_grid(grid);
    let seed: Seed = "AAFP26SGQFDAYVCFVE".parse().unwrap();
    c.bench_function("generate", |b| {
        b.iter(|| generate(black_box(&wrapped_input), black_box(seed)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
