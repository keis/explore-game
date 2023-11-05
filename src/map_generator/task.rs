use super::{MapPrototype, ZonePrototype};
use crate::{
    terrain::{Outer, Terrain},
    ExplError,
};
use bevy::prelude::*;
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

pub fn generate_map(seed: Seed) -> Result<MapPrototype, ExplError> {
    info!("Generating map with seed {} ...", seed);
    let mut file = io::BufReader::new(File::open("assets/maps/test.txt")?);
    let input = Grid::<HexagonalGridLayout, Terrain>::load(&mut file)?;
    let wrapped_input = wrap_grid(input);
    let transforms = standard_tile_transforms();
    let template = Template::from_tiles(extract_tiles(&wrapped_input, &transforms));
    let mut generator = Generator::new_with_seed(&template, seed)?;

    while generator.step().is_some() {}
    info!("Generated map!");
    let terrain: Grid<SquareGridLayout, Terrain> = generator.export()?;
    let mut rng = generator.rand();

    let party_position = spiral(terrain.layout.center())
        .find(|&c| {
            terrain
                .get(c)
                .map_or(false, |&terrain| terrain != Terrain::Ocean)
        })
        .ok_or(ExplError::CouldNotPlaceParty)?;

    let portal_position = spiral(
        terrain.layout.center() + *HexCoord::NEIGHBOUR_OFFSETS.choose(&mut rng).unwrap() * 3,
    )
    .find(|&c| {
        terrain.get(c).map_or(false, |&terrain| {
            terrain != Terrain::Ocean && terrain != Terrain::Mountain
        })
    })
    .ok_or(ExplError::CouldNotPlacePortal)?;

    let spawner_position = spiral(
        terrain.layout.center() + *HexCoord::NEIGHBOUR_OFFSETS.choose(&mut rng).unwrap() * 4,
    )
    .find(|&c| {
        terrain.get(c).map_or(false, |&terrain| {
            terrain != Terrain::Ocean && terrain != Terrain::Mountain
        })
    })
    .ok_or(ExplError::CouldNotPlaceSpawner)?;

    let mut prototype = Grid::with_data(
        terrain.layout,
        terrain.iter().map(|(coord, &terrain)| match terrain {
            Terrain::Forest => ZonePrototype {
                terrain,
                random_fill: if coord == portal_position {
                    random_fill(vec![(Vec2::ZERO, 0.3)])
                } else {
                    random_fill(vec![])
                },
                crystals: rng.gen_range(0..8) == 0,
                height_amp: 0.1,
                height_base: 0.0,
                ..default()
            },
            Terrain::Mountain => ZonePrototype {
                terrain,
                height_amp: 0.5,
                height_base: 0.1,
                ..default()
            },
            Terrain::Ocean => ZonePrototype {
                terrain,
                height_amp: -0.2,
                height_base: -0.5,
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
        let neighbour_base: Vec<_> = coord
            .neighbours()
            .map(|neighbour| {
                prototype
                    .get(neighbour)
                    .map(|proto| proto.height_base)
                    .unwrap_or(0.0)
            })
            .collect();
        let zone = &mut prototype[coord];
        zone.outer_amp = Outer::new(&neighbour_amp);
        zone.outer_base = Outer::new(&neighbour_base);
    }
    Ok(MapPrototype {
        tiles: prototype,
        party_position,
        portal_position,
        spawner_position,
    })
}
