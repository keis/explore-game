use super::{Terrain, ZonePrototype};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use expl_hexgrid::{
    layout::{HexagonalGridLayout, SquareGridLayout},
    spiral, Grid, GridLayout, HexCoord,
};
use expl_wfc::{
    tile::extract_tiles, tile::standard_tile_transforms, util::wrap_grid, util::LoadGrid,
    Generator, Seed, Template,
};
use rand::{seq::SliceRandom, Rng};
use std::fs::File;
use std::io;

pub type MapPrototype = Grid<SquareGridLayout, ZonePrototype>;

#[derive(Component)]
pub struct GenerateMapTask(pub Task<Result<MapPrototype, &'static str>>);

#[derive(Resource)]
pub struct MapSeed(pub Seed);

fn random_in_circle<R: Rng>(rng: &mut R, radius: f32) -> Vec2 {
    let max_r = radius * radius;
    let sqrtr = rng.gen_range(0.0f32..max_r).sqrt();
    let angle = rng.gen_range(0.0f32..(2.0 * std::f32::consts::PI));
    Vec2::new(sqrtr * angle.cos(), sqrtr * angle.sin())
}

fn random_fill(fixed: Vec<(Vec2, f32)>) -> Vec<(Vec2, f32)> {
    // Pretty stupid algorithm that simply tries a few random positions and returns whatever didn't
    // overlap
    let mut rng = rand::thread_rng();
    let mut result: Vec<(Vec2, f32)> = vec![];
    for _ in 0..16 {
        let newpos = random_in_circle(&mut rng, 0.8);
        let newradius = rng.gen_range(0.18f32..0.22);
        if !fixed
            .iter()
            .chain(result.iter())
            .any(|(pos, radius)| pos.distance(newpos) < radius + newradius)
        {
            result.push((newpos, newradius));
        }
    }
    result
}

fn generate_map(seed: Seed) -> Result<MapPrototype, &'static str> {
    info!("Generating map with seed {} ...", seed);
    let mut file =
        io::BufReader::new(File::open("assets/maps/test.txt").map_err(|_| "failed to open file")?);
    let input = Grid::<HexagonalGridLayout, Terrain>::load(&mut file)?;
    let wrapped_input = wrap_grid(input);
    let transforms = standard_tile_transforms();
    let template = Template::from_tiles(extract_tiles(&wrapped_input, &transforms));
    let mut generator = Generator::new_with_seed(&template, seed)?;

    while generator.step().is_some() {}
    info!("Generated map!");
    let terrain: Grid<SquareGridLayout, Terrain> = generator.export()?;
    let mut rng = rand::thread_rng();
    let portalcoord = spiral(
        terrain.layout.center() + *HexCoord::NEIGHBOUR_OFFSETS.choose(&mut rng).unwrap() * 3,
    )
    .find(|&c| {
        terrain.get(c).map_or(false, |&terrain| {
            terrain != Terrain::Ocean && terrain != Terrain::Mountain
        })
    })
    .ok_or("could not place portal")?;

    let mut prototype = Grid::with_data(
        terrain.layout,
        terrain.iter().map(|(coord, &terrain)| match terrain {
            Terrain::Forest => ZonePrototype {
                terrain,
                random_fill: if coord == portalcoord {
                    random_fill(vec![(Vec2::ZERO, 0.3)])
                } else {
                    random_fill(vec![])
                },
                crystals: rng.gen_range(0..8) == 0,
                portal: coord == portalcoord,
                height_amp: 0.1,
                ..default()
            },
            Terrain::Mountain => ZonePrototype {
                terrain,
                portal: coord == portalcoord,
                height_amp: 0.5,
                ..default()
            },
            _ => ZonePrototype {
                terrain,
                ..default()
            },
        }),
    );
    let layout = prototype.layout;
    for coord in layout.iter() {
        let neighbour_amp: Vec<_> = coord
            .neighbours()
            .map(|neighbour| {
                prototype
                    .get(neighbour)
                    .map(|proto| proto.height_amp)
                    .unwrap_or(0.0)
            })
            .collect();
        let mut zone = &mut prototype[coord];
        for i in 0..=5 {
            zone.outer_amp[i] = zone
                .height_amp
                .min(neighbour_amp[i])
                .min(neighbour_amp[(i + 1) % 6]);
        }
    }
    Ok(prototype)
}

pub fn start_map_generation(mut commands: Commands, seed_res: Res<MapSeed>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let seed = seed_res.into_inner().0;
    let task = thread_pool.spawn(async move { generate_map(seed) });
    commands.spawn(GenerateMapTask(task));
}
