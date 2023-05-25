use criterion::{black_box, criterion_group, criterion_main, Criterion};
use expl_hexgrid::{
    layout::{HexagonalGridLayout, SquareGridLayout},
    Grid,
};
use expl_wfc::{
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, LoadGrid},
    Generator, Seed, Template,
};
use pprof::criterion::{Output, PProfProfiler};
use std::{fs::File, io, time::Duration};

fn sample_grid() -> Result<Grid<HexagonalGridLayout, char>, &'static str> {
    let mut file =
        io::BufReader::new(File::open("res/test.txt").map_err(|_| "failed to open file")?);
    Grid::<HexagonalGridLayout, char>::load(&mut file).map_err(|_| "infallible")
}

fn template_from_grid(input: &Grid<HexagonalGridLayout, char>) -> Template<char> {
    let transforms = standard_tile_transforms();
    Template::from_tiles(extract_tiles(input, &transforms))
}

fn generate_hexagonal_from_template(
    template: &Template<char>,
    seed: Seed,
) -> Grid<HexagonalGridLayout, char> {
    let mut generator = Generator::new_with_seed(template, seed).unwrap();
    while generator.step().is_some() {}
    generator.export().unwrap()
}

fn generate_square_from_template(
    template: &Template<char>,
    seed: Seed,
) -> Grid<SquareGridLayout, char> {
    let mut generator = Generator::new_with_seed(template, seed).unwrap();
    while generator.step().is_some() {}
    generator.export().unwrap()
}

fn generate_full(
    grid: &Grid<HexagonalGridLayout, char>,
    seed: Seed,
) -> Grid<HexagonalGridLayout, char> {
    let template = template_from_grid(grid);
    generate_hexagonal_from_template(&template, seed)
}

pub fn benchmark_load_template(c: &mut Criterion) {
    let grid = sample_grid().unwrap();
    let wrapped_input = wrap_grid(grid);
    c.bench_function("load_template", |b| {
        b.iter(|| template_from_grid(black_box(&wrapped_input)))
    });
}

pub fn benchmark_generate_full(c: &mut Criterion) {
    let grid = sample_grid().unwrap();
    let wrapped_input = wrap_grid(grid);
    let seed: Seed = "AAFP26SGQFDAYVCFVE".parse().unwrap();
    c.bench_function("generate_full", |b| {
        b.iter(|| generate_full(black_box(&wrapped_input), black_box(seed)))
    });
}

pub fn benchmark_generate_only(c: &mut Criterion) {
    let grid = sample_grid().unwrap();
    let wrapped_input = wrap_grid(grid);
    let seed: Seed = "AAFP26SGQFDAYVCFVE".parse().unwrap();
    let template = template_from_grid(&wrapped_input);
    c.bench_function("generate_only", |b| {
        b.iter(|| generate_hexagonal_from_template(black_box(&template), black_box(seed)))
    });
}

pub fn benchmark_generate_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("slow");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    let grid = sample_grid().unwrap();
    let wrapped_input = wrap_grid(grid);
    let seed: Seed = "AF4HR7O3U6VZGHS5TOCQ".parse().unwrap();
    let template = template_from_grid(&wrapped_input);
    group.bench_function("generate_large", |b| {
        b.iter(|| generate_square_from_template(black_box(&template), black_box(seed)))
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_load_template, benchmark_generate_only, benchmark_generate_full, benchmark_generate_large
);
criterion_main!(benches);
