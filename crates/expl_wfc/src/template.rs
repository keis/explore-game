use super::{tile::Tile, TileId};
use expl_hexgrid::{GridLayout, HexCoord, Neighbours};
use fixedbitset::FixedBitSet;
use std::collections::BinaryHeap;
use std::hash::Hash;

pub struct TileDetails<Item> {
    contribution: Item,
    compatible: Neighbours<FixedBitSet>,
}

/// A Template holds the rules for how tiles can be combined
pub struct Template<Item> {
    details: Vec<TileDetails<Item>>,
}

#[derive(Debug)]
pub struct TemplateStats {
    pub size: u32,
    pub mean: f32,
    pub stddev: f32,
    pub min: u32,
    pub max: u32,
}

impl<'a, Item> Template<Item>
where
    Item: Copy + PartialEq + Ord + Hash + 'a,
{
    pub fn from_tiles<Layout, Iter>(iter: Iter) -> Self
    where
        Layout: GridLayout + 'a,
        Iter: IntoIterator<Item = Tile<'a, Layout, Item>>,
    {
        let tiles = iter
            .into_iter()
            .collect::<BinaryHeap<_>>()
            .into_sorted_vec();
        let details = tiles
            .iter()
            .map(|tile| TileDetails {
                contribution: tile[HexCoord::ZERO],
                compatible: Neighbours::from_fn(|offset| {
                    let mut bitset = FixedBitSet::with_capacity(tiles.len());
                    bitset.extend(
                        tiles
                            .iter()
                            .enumerate()
                            .filter(|(_, other)| tile.compatible_with(other, offset))
                            .map(|(id, _)| id as TileId),
                    );
                    bitset
                }),
            })
            .collect();
        Self { details }
    }

    pub fn compatible_tiles(
        &self,
        tile_id: TileId,
    ) -> impl '_ + Iterator<Item = (HexCoord, &FixedBitSet)> {
        self.details[tile_id].compatible.iter()
    }

    pub fn available_tiles(&self) -> usize {
        self.details.len()
    }

    pub fn contribution(&self, tile_id: TileId) -> Item {
        self.details[tile_id].contribution
    }

    pub fn stats(&self) -> TemplateStats {
        let connections: Vec<_> = self
            .details
            .iter()
            .flat_map(|d| d.compatible.iter_values().map(|c| c.len() as u32))
            .collect();
        let mean = connections.iter().sum::<u32>() as f32 / connections.len() as f32;
        let stddev = connections
            .iter()
            .map(|&c| (c as f32 - mean).powf(2.0))
            .sum::<f32>()
            / connections.len() as f32;
        TemplateStats {
            size: self.details.len() as u32,
            mean,
            stddev,
            max: *connections.iter().max().unwrap(),
            min: *connections.iter().min().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Template;
    use crate::{
        tile::{
            extract_tiles,
            tests::{sample_map, standard_transforms},
        },
        TileId,
    };
    use expl_hexgrid::{layout::HexagonalGridLayout, Grid, TransformMatrix};
    use rstest::*;

    #[rstest]
    fn from_tiles(
        standard_transforms: Vec<TransformMatrix>,
        sample_map: Grid<HexagonalGridLayout, char>,
    ) {
        let tiles = extract_tiles(&sample_map, &standard_transforms);
        let ntiles = tiles.len();
        let template = Template::from_tiles(tiles);

        assert_eq!(template.available_tiles(), ntiles);
        assert_eq!(template.contribution(0 as TileId), '%');
        assert_eq!(template.contribution(1 as TileId), '^');
        assert_eq!(template.contribution(2 as TileId), '%');
        assert_eq!(template.contribution(3 as TileId), '^');
        assert_eq!(template.contribution(4 as TileId), '%');
        assert_eq!(template.contribution(5 as TileId), '^');
    }
}
