use super::WFCError;
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

pub trait LoadGridWith: Sized {
    type Error;
    type Item;

    fn load_with<R: BufRead, F, E>(buf: &mut R, load_item: F) -> Result<Self, Self::Error>
    where
        F: Fn(char) -> Result<Self::Item, E>;
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
        for r in -(self.layout.radius - 1)..(self.layout.radius) {
            write!(writer, "\n{}", " ".repeat(r.abs().try_into().unwrap()))?;
            for q in (-self.layout.radius - r.min(0) + 1)..(self.layout.radius - r.max(0)) {
                write!(writer, " {}", dump_item(&self[(q, r).into()]))?;
            }
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
        for r in 0..self.layout.height {
            writeln!(writer)?;
            if r % 2 == 1 {
                write!(writer, " ")?;
            }
            for q in (-r / 2)..(self.layout.width - (r / 2)) {
                write!(writer, " {}", dump_item(&self[(q, r).into()]))?;
            }
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

impl<Item: Clone + Default> LoadGridWith for Grid<HexagonalGridLayout, Item> {
    type Error = WFCError;
    type Item = Item;

    fn load_with<R: BufRead, F, E>(buf: &mut R, load_item: F) -> Result<Self, Self::Error>
    where
        F: Fn(char) -> Result<Self::Item, E>,
    {
        let lines: Vec<_> = buf.lines().collect::<Result<_, _>>()?;
        let layout = HexagonalGridLayout {
            radius: (lines.len() / 2 + 1) as i32,
        };
        let data: Vec<_> = lines
            .iter()
            .enumerate()
            .flat_map(|(lineno, line)| {
                let r = lineno as i32 - layout.radius + 1;
                line.chars()
                    .skip(((lineno + 1) as i32 - layout.radius).unsigned_abs() as usize + 1)
                    .step_by(2)
                    .zip(std::iter::repeat(r))
                    .enumerate()
                    .map(|(colno, (c, r))| {
                        load_item(c)
                            .map(|d| {
                                (
                                    HexCoord::new(-layout.radius - r.min(0) + colno as i32 + 1, r),
                                    d,
                                )
                            })
                            .map_err(|_| WFCError::CellParseError)
                    })
            })
            .collect::<Result<_, _>>()?;
        let mut grid = Self::new(layout);
        grid.extend(data);
        Ok(grid)
    }
}

impl<Item: TryFrom<char> + Clone + Default> LoadGrid for Grid<HexagonalGridLayout, Item> {
    type Error = WFCError;

    fn load<R: BufRead>(buf: &mut R) -> Result<Self, Self::Error> {
        LoadGridWith::load_with(buf, |char| char.try_into())
    }
}

pub fn wrap_grid<Item: Default + Clone + Copy>(
    grid: Grid<HexagonalGridLayout, Item>,
) -> Grid<HexagonalGridLayout, Item> {
    let layout = HexagonalGridLayout {
        radius: grid.layout.radius + 1,
    };
    let mut wrapped = Grid::new(layout);
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
    use super::{DumpGrid, LoadGrid, LoadGridWith};
    use crate::WFCError;
    use expl_hexgrid::HexCoord;
    use expl_hexgrid::{layout::HexagonalGridLayout, Grid};
    use std::fs::File;
    use std::io::{BufReader, BufWriter};

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
    pub enum Terrain {
        #[default]
        Ocean,
        Mountain,
        Forest,
    }

    impl From<Terrain> for char {
        fn from(terrain: Terrain) -> Self {
            match terrain {
                Terrain::Forest => '%',
                Terrain::Mountain => '^',
                Terrain::Ocean => '~',
            }
        }
    }

    impl TryFrom<char> for Terrain {
        type Error = WFCError;

        fn try_from(c: char) -> Result<Terrain, Self::Error> {
            match c {
                '%' => Ok(Terrain::Forest),
                '^' => Ok(Terrain::Mountain),
                '~' => Ok(Terrain::Ocean),
                _ => Err(WFCError::UnknownError),
            }
        }
    }

    #[test]
    fn load_sample_grid() -> Result<(), WFCError> {
        let mut file = BufReader::new(File::open("res/test.txt")?);
        let grid = Grid::<HexagonalGridLayout, Terrain>::load(&mut file)?;

        assert_eq!(grid.layout.radius, 5);
        assert_eq!(grid[HexCoord::ZERO], Terrain::Mountain);

        let mut writer = BufWriter::new(Vec::new());
        grid.dump(&mut writer).unwrap();
        let string = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(string.len(), 152);

        Ok(())
    }

    #[test]
    fn load_sample_grid_transformed() -> Result<(), WFCError> {
        let mut file = BufReader::new(File::open("res/test.txt")?);
        let grid = Grid::<HexagonalGridLayout, Terrain>::load_with(&mut file, |_| {
            Ok::<_, &str>(Terrain::Forest)
        })?;

        assert_eq!(grid.layout.radius, 5);
        assert_eq!(grid[HexCoord::ZERO], Terrain::Forest);

        Ok(())
    }
}
