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

impl<Item: TryFrom<char> + Clone + Default> LoadGrid for Grid<HexagonalGridLayout, Item>
where
    Item::Error: std::error::Error + 'static,
{
    type Error = Box<dyn std::error::Error>;

    fn load<R: BufRead>(buf: &mut R) -> Result<Self, Self::Error> {
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
                    .enumerate()
                    .map(move |(colno, c)| {
                        c.try_into().map(|d| {
                            (
                                HexCoord::new(-layout.radius - r.min(0) + colno as i32 + 1, r),
                                d,
                            )
                        })
                    })
            })
            .collect::<Result<_, _>>()?;
        let mut grid = Self::new(layout);
        grid.extend(data);
        Ok(grid)
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
    use super::{DumpGrid, LoadGrid};
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

    #[derive(Debug)]
    pub struct Error(&'static str);

    impl std::error::Error for Error {}

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl TryFrom<char> for Terrain {
        type Error = Error;

        fn try_from(c: char) -> Result<Terrain, Self::Error> {
            match c {
                '%' => Ok(Terrain::Forest),
                '^' => Ok(Terrain::Mountain),
                '~' => Ok(Terrain::Ocean),
                _ => Err(Error("Unknown terrain character")),
            }
        }
    }

    #[test]
    fn load_sample_grid() {
        let mut file = BufReader::new(File::open("res/test.txt").unwrap());
        let grid = Grid::<HexagonalGridLayout, Terrain>::load(&mut file).unwrap();
        assert_eq!(grid.layout.radius, 5);
        let mut writer = BufWriter::new(Vec::new());
        grid.dump(&mut writer).unwrap();
        let string = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(string.len(), 152);
    }
}
