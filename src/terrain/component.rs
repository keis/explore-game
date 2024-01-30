use super::asset::Terrain;
use bevy::prelude::*;
use expl_codex::{Codex, Id};
use expl_hexgrid::Neighbours;
use noisy_bevy::simplex_noise_2d;
use std::cmp::min;

#[derive(Component, Reflect, Default, Deref)]
#[reflect(Component)]
pub struct TerrainId(pub Id<Terrain>);

impl TerrainId {
    pub fn from_tag(tag: &str) -> Self {
        Self(Id::from_tag(tag))
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct CrystalDeposit {
    pub amount: u8,
}

impl CrystalDeposit {
    pub fn take(&mut self) -> u8 {
        let take = min(self.amount, 10);
        self.amount -= take;
        take
    }
}

#[derive(Reflect, Default, Debug)]
pub struct ZoneDecorationDetail {
    pub relative: Vec2,
    pub scale: f32,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct ZoneDecorations {
    pub crystal_detail: Option<ZoneDecorationDetail>,
    pub tree_details: Vec<ZoneDecorationDetail>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ZoneDecorationCrystals;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ZoneDecorationTree;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Water;

#[derive(Component, Default, Deref, DerefMut, Reflect, Copy, Clone)]
pub struct OuterTerrain(pub Neighbours<Id<Terrain>>);

#[derive(Component, Default, Deref, DerefMut, Reflect, Copy, Clone)]
pub struct OuterVisible(pub Neighbours<bool>);

impl OuterVisible {
    pub fn with_data(data: [bool; 6]) -> Self {
        Self(Neighbours::new(data))
    }

    pub fn all_visible() -> Self {
        Self(Neighbours::new([true; 6]))
    }
}

#[derive(Reflect, Default, Copy, Clone)]
pub struct Height {
    pub height_amp: f32,
    pub height_base: f32,
    #[reflect(skip_serializing)]
    pub outer_amp: Neighbours<f32>,
    #[reflect(skip_serializing)]
    pub outer_base: Neighbours<f32>,
}

impl Height {
    pub fn new(
        terrain_codex: &Codex<Terrain>,
        terrain_id: Id<Terrain>,
        neighbours: &Neighbours<Id<Terrain>>,
    ) -> Self {
        Self {
            height_amp: terrain_codex[&terrain_id].height_amp,
            height_base: terrain_codex[&terrain_id].height_base,
            outer_amp: neighbours.map(|terrain| terrain_codex[&terrain].height_amp),
            outer_base: neighbours.map(|terrain| terrain_codex[&terrain].height_base),
        }
    }

    fn clamp(min_value: f32, max_value: f32) -> f32 {
        if min_value < 0.0 && max_value < 0.0 {
            max_value
        } else if min_value < 0.0 {
            0.0
        } else {
            min_value
        }
    }

    fn compute_corner(outer: &Neighbours<f32>, self_value: f32, idx: usize) -> f32 {
        let a = outer[idx];
        let b = outer[(idx + 1) % 6];

        let min_value = self_value.min(a).min(b);
        let max_value = self_value.max(a).max(b);

        Height::clamp(min_value, max_value)
    }

    fn compute_edge(outer: &Neighbours<f32>, self_value: f32, idx: usize) -> f32 {
        let a = outer[idx];

        let min_value = self_value.min(a);
        let max_value = self_value.max(a);

        Height::clamp(min_value, max_value)
    }

    fn corner(&self, idx: usize) -> (f32, f32) {
        (
            Height::compute_corner(&self.outer_amp, self.height_amp, idx),
            Height::compute_corner(&self.outer_base, self.height_base, idx),
        )
    }

    fn edge(&self, idx: usize) -> (f32, f32) {
        (
            Height::compute_edge(&self.outer_amp, self.height_amp, idx),
            Height::compute_edge(&self.outer_base, self.height_base, idx),
        )
    }

    pub(super) fn amp_and_base(&self, local_position: Vec2) -> (f32, f32) {
        // TODO: This could could be simplified by using the fact that the hexagon is symmetrical
        // in both X and Y
        if local_position.y >= 0.9 {
            // South corner
            self.corner(1)
        } else if local_position.y <= -0.9 {
            // North Corner
            self.corner(4)
        } else if local_position.x > 0.0 {
            if local_position.y > local_position.x * -0.5 + 0.9 {
                // South-East Corner or South-East Edge
                if local_position.x > 0.8 {
                    self.corner(0)
                } else {
                    self.edge(1)
                }
            } else if local_position.y < local_position.x * 0.5 - 0.9 {
                // North-East Corner or North-East Edge
                if local_position.x > 0.8 {
                    self.corner(5)
                } else {
                    self.edge(5)
                }
            } else if local_position.x > 0.8 {
                // East Edge
                self.edge(0)
            } else {
                // Internal
                (self.height_amp, self.height_base)
            }
        } else if local_position.x < 0.0 {
            if local_position.y > -local_position.x * -0.5 + 0.9 {
                // South-West Corner or South-West Edge
                if local_position.x < -0.8 {
                    self.corner(2)
                } else {
                    self.edge(2)
                }
            } else if local_position.y < -local_position.x * 0.5 - 0.9 {
                // North-West Corner or North-West Edge
                if local_position.x < -0.8 {
                    self.corner(3)
                } else {
                    self.edge(4)
                }
            } else if local_position.x < 0.8 {
                // West Edge
                self.edge(3)
            } else {
                // Internal
                (self.height_amp, self.height_base)
            }
        } else {
            (self.height_amp, self.height_base)
        }
    }

    pub fn height_at(&self, local_position: Vec2, world_position: Vec2) -> f32 {
        let (amp, base) = self.amp_and_base(local_position);
        let noise = (1.0 + simplex_noise_2d(world_position)) / 2.0;
        base + noise * amp
    }
}
