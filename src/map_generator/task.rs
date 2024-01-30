use super::{MapPrototype, MapTemplate, ZonePrototype};
use crate::{
    terrain::{Terrain, TerrainDecoration},
    ExplError,
};
use bevy::prelude::*;
use expl_codex::{Codex, Id};
use expl_hexgrid::{layout::SquareGridLayout, spiral, Grid, GridLayout, HexCoord};
use expl_wfc::{Generator, Seed};
use rand::{seq::SliceRandom, Rng};

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

pub fn generate_map(
    terrain_codex: &Codex<Terrain>,
    template: &MapTemplate,
    seed: Seed,
) -> Result<MapPrototype, ExplError> {
    info!("Generating map with seed {} ...", seed);
    let mut generator = Generator::new_with_seed(template, seed)?;

    while generator.step().is_some() {}
    info!("Generated map!");
    let terrain: Grid<SquareGridLayout, Id<Terrain>> = generator.export()?;
    let mut rng = generator.rand();

    let party_position = spiral(terrain.layout.center())
        .find(|&c| {
            terrain
                .get(c)
                .map_or(false, |terrain| terrain_codex[terrain].allow_walking)
        })
        .ok_or(ExplError::CouldNotPlaceParty)?;

    let portal_position = spiral(
        terrain.layout.center() + *HexCoord::NEIGHBOUR_OFFSETS.choose(&mut rng).unwrap() * 3,
    )
    .find(|&c| {
        terrain
            .get(c)
            .map_or(false, |terrain| terrain_codex[terrain].allow_structure)
    })
    .ok_or(ExplError::CouldNotPlacePortal)?;

    let spawner_position = spiral(
        terrain.layout.center() + *HexCoord::NEIGHBOUR_OFFSETS.choose(&mut rng).unwrap() * 4,
    )
    .find(|&c| {
        terrain
            .get(c)
            .map_or(false, |terrain| terrain_codex[terrain].allow_structure)
    })
    .ok_or(ExplError::CouldNotPlaceSpawner)?;

    let tiles = Grid::with_data(
        terrain.layout,
        terrain.iter().map(|(coord, &terrain)| {
            let terrain_data = &terrain_codex[&terrain];
            let with_trees = terrain_data.decoration.contains(&TerrainDecoration::Tree);
            let with_crystals = terrain_data
                .decoration
                .contains(&TerrainDecoration::Crystal);
            let random_fill = if with_trees || with_crystals {
                if coord == portal_position {
                    random_fill(vec![(Vec2::ZERO, 0.3)])
                } else {
                    random_fill(vec![])
                }
            } else {
                Vec::default()
            };
            let crystals = with_crystals && rng.gen_range(0..8) == 0;
            ZonePrototype {
                terrain,
                random_fill,
                crystals,
            }
        }),
    );
    Ok(MapPrototype {
        tiles,
        party_position,
        portal_position,
        spawner_position,
    })
}
