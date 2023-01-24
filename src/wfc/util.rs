use crate::hexgrid::{layout::HexagonalGridLayout, Grid, GridLayout};
use std::io;
use std::io::BufRead;

pub fn dump_grid_with<Item, W: io::Write, F>(
    grid: &Grid<HexagonalGridLayout, Item>,
    writer: &mut W,
    dump_item: F,
) -> io::Result<()>
where
    F: Fn(&Item) -> char,
{
    let mut lastr = 0;
    for coord in grid.layout.iter() {
        if coord.r != lastr {
            write!(
                writer,
                "\n{}",
                " ".repeat(coord.r.abs().try_into().unwrap())
            )?;
            lastr = coord.r;
        }
        write!(writer, " {}", dump_item(&grid[coord]))?;
    }
    writeln!(writer)?;
    Ok(())
}

pub fn dump_grid<Item: Into<char> + Copy, W: io::Write>(
    grid: &Grid<HexagonalGridLayout, Item>,
    writer: &mut W,
) -> io::Result<()> {
    dump_grid_with(grid, writer, |&item| item.into())
}

pub fn load_grid<Item: TryFrom<char> + Clone, R: BufRead>(
    buf: &mut R,
) -> Result<Grid<HexagonalGridLayout, Item>, Item::Error> {
    let lines: Vec<_> = buf.lines().map(|l| l.unwrap()).collect();
    let layout = HexagonalGridLayout {
        radius: (lines.len() / 2 + 1) as i32,
    };
    let data = lines
        .iter()
        .enumerate()
        .flat_map(|(i, l)| {
            l.chars()
                .skip(((i + 1) as i32 - layout.radius).unsigned_abs() as usize + 1)
                .step_by(2)
                .map(|c| c.try_into())
        })
        .collect::<Result<_, _>>()?;

    Ok(Grid { layout, data })
}

#[cfg(test)]
mod tests {
    use super::{dump_grid, load_grid};
    use crate::zone::Terrain;
    use std::fs::File;
    use std::io::{BufReader, BufWriter};

    #[test]
    fn load_sample_grid() {
        let mut file = BufReader::new(File::open("assets/maps/test.txt").unwrap());
        let grid = load_grid::<Terrain, _>(&mut file).unwrap();
        assert_eq!(grid.layout.radius, 5);
        let mut writer = BufWriter::new(Vec::new());
        dump_grid(&grid, &mut writer).unwrap();
        let string = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(string.len(), 152);
    }
}
