use super::component::*;
use crate::{assets::MainAssets, floating_text::FloatingTextSource};
use bevy::prelude::*;
use bevy_sprite3d::{Sprite3dBuilder, Sprite3dBundle, Sprite3dParams};
use expl_hexgrid::HexCoord;
use smallvec::SmallVec;

#[derive(Bundle)]
pub struct CombatBundle {
    combat: Combat,
    sprite3d: Sprite3dBundle,
    floating_text_source: FloatingTextSource,
    transform: Transform,
}

pub type CombatParams<'w, 's> = (Res<'w, MainAssets>, Sprite3dParams<'w, 's>);

impl CombatBundle {
    pub fn new(
        (main_assets, sprite_params): &mut CombatParams,
        position: HexCoord,
        initiative_order: SmallVec<[Entity; 8]>,
    ) -> Self {
        Self {
            combat: Combat {
                position,
                initiative: 0,
                initiative_order,
            },
            sprite3d: Sprite3dBuilder {
                image: main_assets.swords_emblem_icon.clone(),
                pixels_per_metre: 400.0,
                ..default()
            }
            .bundle(sprite_params),
            transform: Transform::from_translation(Vec3::from(position) + Vec3::new(0.0, 1.0, 0.0)),
            floating_text_source: FloatingTextSource::default(),
        }
    }
}
