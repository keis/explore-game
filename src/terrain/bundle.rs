use super::{asset::*, component::*};
use crate::{
    map::{Fog, HexCoord, MapPosition},
    map_generator::ZonePrototype,
    material::{TerrainMaterial, WaterMaterial, ZoneMaterial},
};
use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::prelude::{Pickable, PickingInteraction};
use glam::Vec3Swizzles;

pub type ZoneDecorationParams<'w> = ResMut<'w, Assets<TerrainMaterial>>;

#[derive(Bundle)]
pub struct ZoneDecorationBundle<Tag: Component> {
    fog: Fog,
    tag: Tag,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

#[allow(clippy::too_many_arguments)]
impl<Tag: Component> ZoneDecorationBundle<Tag> {
    pub fn new(
        tag: Tag,
        decoration_id: Id<Decoration>,
        terrain_materials: &mut ZoneDecorationParams,
        decoration_codex: &Codex<Decoration>,
        height: &Height,
        fog: &Fog,
        position: HexCoord,
        detail: &ZoneDecorationDetail,
    ) -> Self {
        let decoration = &decoration_codex[&decoration_id];
        Self {
            fog: *fog,
            tag,
            material_mesh_bundle: MaterialMeshBundle {
                mesh: decoration.mesh.clone(),
                material: terrain_materials.add(TerrainMaterial::from_decoration(
                    decoration_codex,
                    &decoration_id,
                    fog,
                )),
                visibility: if fog.explored {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                transform: Transform::from_translation(Vec3::new(
                    detail.relative.x,
                    height.height_at(detail.relative, Vec3::from(position).xz() + detail.relative),
                    detail.relative.y,
                ))
                .with_scale(Vec3::splat(detail.scale * decoration.scale)),
                ..default()
            },
        }
    }
}

pub type WaterParams<'w> = (Res<'w, HexAssets>, ResMut<'w, Assets<WaterMaterial>>);

#[derive(Bundle, Default)]
pub struct WaterBundle {
    water: Water,
    fog: Fog,
    not_shadow_caster: NotShadowCaster,
    material_mesh_bundle: MaterialMeshBundle<WaterMaterial>,
}

impl WaterBundle {
    pub fn new((hex_assets, water_materials): &mut WaterParams) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: water_materials.add(WaterMaterial {
                    color: Color::rgba(0.1, 0.1, 0.8, 0.4),
                }),
                transform: Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)),
                ..default()
            },
            ..default()
        }
    }
}

pub type ZoneParams<'w> = (Res<'w, HexAssets>, ResMut<'w, Assets<ZoneMaterial>>);

#[derive(Bundle, Default)]
pub struct ZoneBundle {
    terrain: TerrainId,
    height: Height,
    fog: Fog,
    position: MapPosition,
    zone_decorations: ZoneDecorations,
}

#[derive(Bundle, Default)]
pub struct ZoneFluffBundle {
    pickable: Pickable,
    interaction: PickingInteraction,
    not_shadow_caster: NotShadowCaster,
    material_mesh_bundle: MaterialMeshBundle<ZoneMaterial>,
    outer_visible: OuterVisible,
}

impl ZoneBundle {
    pub fn new(position: HexCoord, prototype: &ZonePrototype) -> Self {
        let terrain = TerrainId(prototype.terrain);
        let height = Height {
            height_amp: prototype.height_amp,
            height_base: prototype.height_base,
            outer_amp: prototype.outer_amp,
            outer_base: prototype.outer_base,
        };
        let mut filliter = prototype.random_fill.iter();
        Self {
            position: MapPosition(position),
            terrain,
            height,
            zone_decorations: ZoneDecorations {
                crystal_detail: if prototype.crystals {
                    filliter
                        .next()
                        .map(|&(relative, scale)| ZoneDecorationDetail { relative, scale })
                } else {
                    None
                },
                tree_details: filliter
                    .map(|&(relative, scale)| ZoneDecorationDetail { relative, scale })
                    .collect(),
            },
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        zone_params: &mut ZoneParams,
        terrain_codex: &Codex<Terrain>,
    ) -> (Self, ZoneFluffBundle) {
        let outer_visible = OuterVisible::default();
        let fluff = ZoneFluffBundle::new(
            zone_params,
            terrain_codex,
            &self.position,
            &self.terrain,
            &self.height,
            &self.fog,
            outer_visible,
        );
        (self, fluff)
    }
}

impl ZoneFluffBundle {
    pub fn new(
        (hex_assets, zone_materials): &mut ZoneParams,
        terrain_codex: &Codex<Terrain>,
        position: &MapPosition,
        terrain: &TerrainId,
        height: &Height,
        fog: &Fog,
        outer_visible: OuterVisible,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: zone_materials.add(ZoneMaterial::new(
                    terrain_codex,
                    terrain,
                    height,
                    fog,
                    &outer_visible,
                )),
                transform: Transform::from_translation(position.0.into()),
                ..default()
            },
            outer_visible,
            ..default()
        }
    }
}
