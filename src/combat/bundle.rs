use super::component::*;
use crate::{assets::MainAssets, floating_text::FloatingTextSource, map::HexCoord};
use bevy::prelude::*;
use bevy_sprite3d::{Sprite3d, Sprite3dBundle, Sprite3dParams};
use smallvec::SmallVec;

#[derive(Bundle)]
pub struct CombatBundle {
    combat: Combat,
    sprite3d: Sprite3dBundle,
    floating_text_source: FloatingTextSource,
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
            sprite3d: Sprite3d {
                image: main_assets.swords_emblem_icon.clone(),
                pixels_per_metre: 400.0,
                transform: Transform::from_translation(
                    Vec3::from(position) + Vec3::new(0.0, 1.0, 0.0),
                ),
                ..default()
            }
            .bundle(sprite_params),
            floating_text_source: FloatingTextSource::default(),
        }
    }
}
