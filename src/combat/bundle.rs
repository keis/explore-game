use super::component::*;
use crate::{assets::MainAssets, floating_text::FloatingTextSource};
use bevy::prelude::*;
use bevy_sprite3d::Sprite3d;
use expl_hexgrid::HexCoord;
use smallvec::SmallVec;

#[derive(Bundle)]
pub struct CombatBundle {
    combat: Combat,
    sprite: Sprite,
    sprite3d: Sprite3d,
    floating_text_source: FloatingTextSource,
    transform: Transform,
}

impl CombatBundle {
    pub fn new(
        main_assets: &Res<MainAssets>,
        position: HexCoord,
        initiative_order: SmallVec<[Entity; 8]>,
    ) -> Self {
        Self {
            combat: Combat {
                position,
                initiative: 0,
                initiative_order,
            },
            sprite: Sprite {
                image: main_assets.swords_emblem_icon.clone(),
                ..default()
            },
            sprite3d: Sprite3d {
                pixels_per_metre: 400.0,
                ..default()
            },
            transform: Transform::from_translation(Vec3::from(position) + Vec3::new(0.0, 1.0, 0.0)),
            floating_text_source: FloatingTextSource::default(),
        }
    }
}
