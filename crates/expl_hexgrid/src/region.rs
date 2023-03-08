use super::HexCoord;

#[derive(Debug)]
pub struct Region(pub Vec<HexCoord>);

impl FromIterator<HexCoord> for Region {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = HexCoord>,
    {
        Region(Vec::<HexCoord>::from_iter(iter))
    }
}
