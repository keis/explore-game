use explore_game::{
    hexgrid::layout::HexagonalGridLayout,
    hexgrid::{ring, Grid, GridLayout, HexCoord},
    wfc::cell::Cell,
    wfc::generator::Generator,
    wfc::template::Template,
    wfc::tile::{extract_tiles, standard_tile_transforms, Tile},
    wfc::TileId,
    zone::Terrain,
};

fn sample_map() -> Grid<HexagonalGridLayout, Terrain> {
    let layout = HexagonalGridLayout { radius: 5 };
    Grid {
        layout,
        data: vec![
            // Row 1
            Terrain::Mountain,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Ocean,
            Terrain::Ocean,
            // Row 2
            Terrain::Mountain,
            Terrain::Mountain,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Ocean,
            Terrain::Forest,
            // Row 3
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            // Row 4
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Mountain,
            Terrain::Mountain,
            Terrain::Forest,
            Terrain::Forest,
            // Row 5
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Mountain,
            Terrain::Mountain,
            Terrain::Mountain,
            Terrain::Mountain,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            // Row 6
            Terrain::Forest,
            Terrain::Mountain,
            Terrain::Forest,
            Terrain::Mountain,
            Terrain::Ocean,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            // Row 7
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Ocean,
            Terrain::Ocean,
            Terrain::Forest,
            Terrain::Forest,
            // Row 8
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Ocean,
            Terrain::Ocean,
            Terrain::Forest,
            Terrain::Forest,
            // Row 9
            Terrain::Forest,
            Terrain::Forest,
            Terrain::Ocean,
            Terrain::Ocean,
            Terrain::Forest,
        ],
    }
}

fn dump_grid<Item, F>(grid: &Grid<HexagonalGridLayout, Item>, display_item: F)
where
    F: Fn(&Item) -> char,
{
    let mut lastr = 0;
    for coord in grid.layout.iter() {
        if coord.r != lastr {
            print!("\n{}", " ".repeat(coord.r.abs().try_into().unwrap()));
            lastr = coord.r;
        }
        print!(" {}", display_item(&grid[coord]));
    }
    println!();
}

fn terrain_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Forest => '%',
        Terrain::Mountain => '^',
        Terrain::Ocean => '~',
    }
}

fn dump_output(
    template: &Template<HexagonalGridLayout, Terrain>,
    grid: &Grid<HexagonalGridLayout, Cell>,
) {
    dump_grid(grid, |cell| match cell {
        Cell::Collapsed(tile) => terrain_char(template.tiles[*tile][HexCoord::ZERO]),
        Cell::Alternatives(_, _) => '?',
    });
}

fn dump_tile(tile: Tile<HexagonalGridLayout, Terrain>) {
    let mut lastr = 0;
    for coord in tile.layout.iter() {
        if coord.r != lastr {
            print!("\n{}", " ".repeat(coord.r.abs().try_into().unwrap()));
            lastr = coord.r;
        }
        print!(" {}", terrain_char(tile[coord]));
    }
    println!();
}

fn wrap_grid(grid: Grid<HexagonalGridLayout, Terrain>) -> Grid<HexagonalGridLayout, Terrain> {
    let layout = HexagonalGridLayout {
        radius: grid.layout.radius + 1,
    };
    let mut wrapped = Grid {
        layout,
        data: vec![Terrain::Ocean; layout.size()],
    };
    for coord in grid.layout.iter() {
        wrapped[coord] = grid[coord];
    }
    for coord in ring(HexCoord::ZERO, grid.layout.radius + 1) {
        wrapped[coord] = grid[grid.layout.wrap(coord)];
    }
    wrapped
}

#[test]
fn test_collapse() {
    let mut rng = rand::thread_rng();
    let input = wrap_grid(sample_map());
    let transforms = standard_tile_transforms();
    let template = Template::from_tiles(extract_tiles(&input, &transforms));
    println!("{:?}", template.stats());
    dump_grid(&input, |&terrain| terrain_char(terrain));

    // Verify all tiles have *some* valid neighbours
    for idx in 0..template.tiles.len() {
        for (_, compatible) in template.compatible_tiles(idx as TileId) {
            if compatible.is_empty() {
                dump_tile(template.tiles[idx as TileId]);
            }
            assert!(!compatible.is_empty());
        }
    }
    let mut generator = Generator::new(&template, HexagonalGridLayout { radius: 5 });
    println!("map size is {:?}", generator.grid.layout.size());
    while generator.step(&mut rng).is_some() {
        for coord in generator.grid.layout.iter() {
            match generator.grid[coord] {
                Cell::Collapsed(_) => {
                    assert_ne!(
                        generator.collapsed.iter().find(|(cc, _, _)| *cc == coord),
                        None
                    );
                }
                Cell::Alternatives(num_alts, _) => {
                    if num_alts < template.tiles.len() {
                        assert_ne!(
                            generator.queue.iter().find(|qc| **qc == coord),
                            None,
                            "expected to find {:?} in queue",
                            coord,
                        );
                    }
                }
            };
        }
    }
    let output = generator.export().unwrap();
    dump_grid(&output, |&terrain| terrain_char(terrain));

    assert_eq!(output.layout.radius, 5);
}
