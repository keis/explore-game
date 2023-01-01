use crate::hexgrid::{GridLayout, HexCoord};
use crate::wfc::tile::Tile;
use crate::wfc::TileId;
use std::collections::HashSet;

pub struct TileDetails {
    compatible: [HashSet<TileId>; 6],
}

/// A Template holds the rules for how tiles can be combined
pub struct Template<'a, Layout: GridLayout, Item> {
    pub tiles: Vec<Tile<'a, Layout, Item>>,
    details: Vec<TileDetails>,
}

#[derive(Debug)]
pub struct TemplateStats {
    pub size: u32,
    pub mean: f32,
    pub stddev: f32,
    pub min: u32,
    pub max: u32,
}

impl<'a, Layout: GridLayout, Item> Template<'a, Layout, Item>
where
    Item: Copy + PartialEq,
{
    pub fn from_tiles<Iter>(iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = Tile<'a, Layout, Item>>,
    {
        let tiles: Vec<_> = iter.into_iter().collect();
        let details = tiles
            .iter()
            .map(|tile| TileDetails {
                compatible: HexCoord::NEIGHBOUR_OFFSETS.map(|offset| {
                    tiles
                        .iter()
                        .enumerate()
                        .filter(|(_, other)| tile.compatible_with(other, offset))
                        .map(|(id, _)| id as TileId)
                        .collect::<HashSet<TileId>>()
                }),
            })
            .collect();
        Self { tiles, details }
    }

    pub fn compatible_tiles(
        &self,
        tile_id: TileId,
    ) -> impl '_ + Iterator<Item = (&HexCoord, &HashSet<TileId>)> {
        HexCoord::NEIGHBOUR_OFFSETS
            .iter()
            .zip(self.details[tile_id].compatible.iter())
    }

    pub fn stats(&self) -> TemplateStats {
        let connections: Vec<_> = self
            .details
            .iter()
            .flat_map(|d| d.compatible.iter().map(|c| c.len() as u32))
            .collect();
        let mean = connections.iter().sum::<u32>() as f32 / connections.len() as f32;
        let stddev = connections
            .iter()
            .map(|&c| (c as f32 - mean).powf(2.0))
            .sum::<f32>()
            / connections.len() as f32;
        TemplateStats {
            size: self.tiles.len() as u32,
            mean,
            stddev,
            max: *connections.iter().max().unwrap(),
            min: *connections.iter().min().unwrap(),
        }
    }
}
