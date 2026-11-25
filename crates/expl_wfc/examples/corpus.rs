//! Runs the generator over a deterministic seed list per map size and reports completion,
//! step/collapse distribution, stale-failure share and time.
//!
//! ```sh
//! cargo run --release -p expl_wfc --example corpus -- 30x24 64 128 --seeds 50
//! cargo run --release -p expl_wfc --example corpus -- 256 --seeds 10 --jobs 4
//! ```
//!
//! Sizes are `WxH` or `N` (an `N`x`N` square). Statistics are taken over all runs; a run that
//! exceeds the collapse budget counts as failed and contributes the work done until then.
//! The stale-failure share is stale failures over all failed selects (rewinds), summed over runs.
use clap::Parser;
use expl_hexgrid::{
    layout::{HexagonalGridLayout, SquareGridLayout},
    Grid,
};
use expl_wfc::{
    tile::{extract_tiles, standard_tile_transforms},
    util::{wrap_grid, LoadGrid},
    Generator, Seed, SeedType, Template, WFCError,
};
use std::{
    fs::File,
    io,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
struct Cli {
    /// Map sizes, `WxH` or `N` for a square
    #[arg(required = true)]
    sizes: Vec<String>,
    /// Number of seeds per size
    #[arg(long, default_value_t = 20, value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..))]
    seeds: usize,
    /// Skip this many seeds of the list
    #[arg(long, default_value_t = 0)]
    offset: usize,
    /// Collapse budget factor K (budget = K x cells); the generator default if omitted
    #[arg(long)]
    budget_factor: Option<usize>,
    /// Number of generations to run in parallel (timings get noisier above 1)
    #[arg(long, default_value_t = 1)]
    jobs: usize,
    /// Print every failed seed
    #[arg(long)]
    list_failures: bool,
    /// Print every run
    #[arg(long)]
    list_runs: bool,
}

fn sample_template() -> Template<char> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/res/test.txt");
    let mut file = io::BufReader::new(File::open(path).unwrap());
    let input = Grid::<HexagonalGridLayout, char>::load(&mut file).unwrap();
    let wrapped_input = wrap_grid(input);
    let transforms = standard_tile_transforms();
    Template::from_tiles(extract_tiles(&wrapped_input, &transforms))
}

fn parse_size(size: &str) -> (u16, u16) {
    match size.split_once('x') {
        Some((w, h)) => (w.parse().unwrap(), h.parse().unwrap()),
        None => {
            let n = size.parse().unwrap();
            (n, n)
        }
    }
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// The deterministic seed list: the n-th seed depends only on the size and n.
fn corpus_seed(width: u16, height: u16, n: usize) -> Seed {
    let base = (u64::from(width) << 16) | u64::from(height);
    Seed::with_rng_seed(
        SeedType::Square(width, height),
        splitmix64(splitmix64(base) ^ n as u64),
    )
}

struct Run {
    seed: Seed,
    completed: bool,
    steps: usize,
    collapses: usize,
    rewinds: usize,
    max_backtrack_depth: usize,
    stale_failures: usize,
    time: Duration,
}

fn run_one(template: &Template<char>, seed: Seed, budget_factor: Option<usize>) -> Run {
    let start = Instant::now();
    let mut generator: Generator<SquareGridLayout, char> =
        Generator::new_with_seed(template, seed).unwrap();
    if let Some(factor) = budget_factor {
        generator = generator.with_budget_factor(factor);
    }
    let completed = loop {
        match generator.step() {
            Ok(Some(())) => {}
            Ok(None) => break true,
            Err(WFCError::BudgetExceeded(_)) | Err(WFCError::HistoryExhausted) => break false,
            Err(error) => panic!("unexpected error: {error}"),
        }
    };
    let time = start.elapsed();
    let stats = *generator.stats();
    Run {
        seed,
        completed,
        steps: stats.steps,
        collapses: stats.collapses,
        rewinds: stats.rewinds,
        max_backtrack_depth: stats.max_backtrack_depth,
        stale_failures: stats.stale_failures,
        time,
    }
}

fn percentile<T: Copy + Ord>(sorted: &[T], p: f64) -> T {
    let index = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[index]
}

fn main() {
    let args = Cli::parse();
    let template = sample_template();
    println!("template: {:?}", template.stats());
    println!(
        "| size | seeds | completed | failed | steps p50 | p90 | max | collapses p50 | p90 | max | steps/cell p50 | max | max depth | stale share | time p50 | max |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");

    for size in &args.sizes {
        let (width, height) = parse_size(size);
        let cells = usize::from(width) * usize::from(height);
        let next = AtomicUsize::new(args.offset);
        let end = args.offset + args.seeds;
        let mut runs: Vec<Run> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..args.jobs.max(1))
                .map(|_| {
                    scope.spawn(|| {
                        let mut runs = Vec::new();
                        loop {
                            let n = next.fetch_add(1, Ordering::Relaxed);
                            if n >= end {
                                break runs;
                            }
                            let seed = corpus_seed(width, height, n);
                            runs.push(run_one(&template, seed, args.budget_factor));
                        }
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|handle| handle.join().unwrap())
                .collect()
        });
        runs.sort_by_key(|run| run.seed.to_string());

        let completed = runs.iter().filter(|run| run.completed).count();
        let mut steps: Vec<_> = runs.iter().map(|run| run.steps).collect();
        let mut collapses: Vec<_> = runs.iter().map(|run| run.collapses).collect();
        let mut times: Vec<_> = runs.iter().map(|run| run.time).collect();
        steps.sort();
        collapses.sort();
        times.sort();
        let rewinds: usize = runs.iter().map(|run| run.rewinds).sum();
        let stale: usize = runs.iter().map(|run| run.stale_failures).sum();
        let stale_share = if rewinds == 0 {
            0.0
        } else {
            stale as f64 / rewinds as f64
        };
        let max_depth = runs
            .iter()
            .map(|run| run.max_backtrack_depth)
            .max()
            .unwrap();
        println!(
            "| {size} | {} | {completed} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {:.2} | {max_depth} | {:.1}% ({stale}/{rewinds}) | {:.2?} | {:.2?} |",
            runs.len(),
            runs.len() - completed,
            percentile(&steps, 0.5),
            percentile(&steps, 0.9),
            steps.last().unwrap(),
            percentile(&collapses, 0.5),
            percentile(&collapses, 0.9),
            collapses.last().unwrap(),
            percentile(&steps, 0.5) as f64 / cells as f64,
            *steps.last().unwrap() as f64 / cells as f64,
            stale_share * 100.0,
            percentile(&times, 0.5),
            times.last().unwrap(),
        );
        if args.list_runs {
            for run in &runs {
                println!(
                    "  {} {} steps={} collapses={} time={:.2?}",
                    if run.completed { "ok    " } else { "failed" },
                    run.seed,
                    run.steps,
                    run.collapses,
                    run.time
                );
            }
        }
        if args.list_failures {
            for run in runs.iter().filter(|run| !run.completed) {
                println!(
                    "  failed {} steps={} collapses={} stale={}/{}",
                    run.seed, run.steps, run.collapses, run.stale_failures, run.rewinds
                );
            }
        }
    }
}
