use super::Terrain;
use crate::hexgrid::{
    layout::{HexagonalGridLayout, SquareGridLayout},
    Grid,
};
use crate::wfc::{
    tile::extract_tiles, tile::standard_tile_transforms, util::wrap_grid, util::LoadGrid,
    Generator, Template,
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use std::fs::File;
use std::io;

#[derive(Component)]
pub struct GenerateMapTask(pub Task<Result<Grid<SquareGridLayout, Terrain>, &'static str>>);

fn generate_map() -> Result<Grid<SquareGridLayout, Terrain>, &'static str> {
    info!("Generating map...");
    let mut file =
        io::BufReader::new(File::open("assets/maps/test.txt").map_err(|_| "failed to open file")?);
    let input = Grid::<HexagonalGridLayout, Terrain>::load(&mut file)?;
    let wrapped_input = wrap_grid(input);
    let transforms = standard_tile_transforms();
    let template = Template::from_tiles(extract_tiles(&wrapped_input, &transforms));
    let mut generator = Generator::new_with_layout(
        &template,
        SquareGridLayout {
            width: 30,
            height: 24,
        },
    );

    while generator.step().is_some() {}
    info!("Generated map!");
    generator.export()
}

pub fn start_map_generation(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { generate_map() });
    commands.spawn(GenerateMapTask(task));
}
