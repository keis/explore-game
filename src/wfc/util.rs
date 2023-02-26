use expl_hexgrid::{
    layout::{HexagonalGridLayout, SquareGridLayout},
    ring, Grid, GridLayout, HexCoord,
};
use std::io;
use std::io::BufRead;

pub trait DumpGridWith {
    type Item;

    fn dump_with<W: io::Write, F>(&self, writer: &mut W, dump_item: F) -> io::Result<()>
    where
        F: Fn(&Self::Item) -> char;
}

pub trait DumpGrid {
    type Item;

    fn dump<W: io::Write>(&self, writer: &mut W) -> io::Result<()>;
}

pub trait LoadGrid: Sized {
    type Error;

    fn load<R: BufRead>(buf: &mut R) -> Result<Self, Self::Error>;
}

impl<Item> DumpGridWith for Grid<HexagonalGridLayout, Item> {
    type Item = Item;

    fn dump_with<W: io::Write, F>(&self, writer: &mut W, dump_item: F) -> io::Result<()>
    where
        F: Fn(&Item) -> char,
    {
        let mut lastr = 0;
        for coord in self.layout.iter() {
            if coord.r != lastr {
                write!(
                    writer,
                    "\n{}",
                    " ".repeat(coord.r.abs().try_into().unwrap())
                )?;
                lastr = coord.r;
            }
            write!(writer, " {}", dump_item(&self[coord]))?;
        }
        writeln!(writer)?;
        Ok(())
    }
}

impl<Item> DumpGridWith for Grid<SquareGridLayout, Item> {
    type Item = Item;

    fn dump_with<W: io::Write, F>(&self, writer: &mut W, dump_item: F) -> io::Result<()>
    where
        F: Fn(&Item) -> char,
    {
        let mut lastr = -1;
        for coord in self.layout.iter() {
            if coord.r != lastr {
                writeln!(writer)?;
                if coord.r % 2 == 1 {
                    write!(writer, " ")?;
                }
                lastr = coord.r;
            }
            write!(writer, " {}", dump_item(&self[coord]))?;
        }
        writeln!(writer)?;
        Ok(())
    }
}

impl<Item: Into<char> + Copy, T: DumpGridWith<Item = Item>> DumpGrid for T {
    type Item = Item;

    fn dump<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.dump_with(writer, |&item| item.into())
    }
}

impl<Item: TryFrom<char> + Clone> LoadGrid for Grid<HexagonalGridLayout, Item> {
    type Error = Item::Error;

    fn load<R: BufRead>(buf: &mut R) -> Result<Self, Self::Error> {
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

        Ok(Self { layout, data })
    }
}

pub fn wrap_grid<Item: Default + Clone + Copy>(
    grid: Grid<HexagonalGridLayout, Item>,
) -> Grid<HexagonalGridLayout, Item> {
    let layout = HexagonalGridLayout {
        radius: grid.layout.radius + 1,
    };
    let mut wrapped = Grid {
        layout,
        data: vec![Item::default(); layout.size()],
    };
    for coord in grid.layout.iter() {
        wrapped[coord] = grid[coord];
    }
    for coord in ring(HexCoord::ZERO, grid.layout.radius + 1) {
        wrapped[coord] = grid[grid.layout.wrap(coord)];
    }
    wrapped
}

#[cfg(test)]
mod tests {
    use super::{DumpGrid, LoadGrid};
    use crate::map::Terrain;
    use expl_hexgrid::{layout::HexagonalGridLayout, Grid};
    use std::fs::File;
    use std::io::{BufReader, BufWriter};

    #[test]
    fn load_sample_grid() {
        let mut file = BufReader::new(File::open("assets/maps/test.txt").unwrap());
        let grid = Grid::<HexagonalGridLayout, Terrain>::load(&mut file).unwrap();
        assert_eq!(grid.layout.radius, 5);
        let mut writer = BufWriter::new(Vec::new());
        grid.dump(&mut writer).unwrap();
        let string = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(string.len(), 152);
    }
}
