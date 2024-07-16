use super::{asset::*, component::*};
use crate::{
    map::{Fog, HexCoord, MapPosition},
    map_generator::ZonePrototype,
    material::{TerrainBuffer, TerrainMaterial, WaterMaterial, ZoneMaterial},
    role::Role,
};
use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::prelude::{Pickable, PickingInteraction};
use expl_codex::{Codex, Id};
use expl_hexgrid::Neighbours;
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
                    color: Color::srgba(0.1, 0.1, 0.8, 0.4),
                }),
                transform: Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)),
                ..default()
            },
            ..default()
        }
    }
}

pub type ZoneParams<'w> = (
    Res<'w, HexAssets>,
    Res<'w, TerrainBuffer>,
    ResMut<'w, Assets<ZoneMaterial>>,
);

#[derive(Default)]
pub struct ZoneRole {
    // Insert
    pickable: Pickable,
    interaction: PickingInteraction,
    not_shadow_caster: NotShadowCaster,
    material_mesh_bundle: MaterialMeshBundle<ZoneMaterial>,
    outer_visible: OuterVisible,
    outer_terrain: OuterTerrain,
}

impl ZoneRole {
    pub fn new(
        (hex_assets, terrain_buffer, zone_materials): &mut ZoneParams,
        position: &MapPosition,
        terrain: &TerrainId,
        fog: &Fog,
        outer_visible: OuterVisible,
        outer_terrain: Neighbours<Id<Terrain>>,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: zone_materials.add(ZoneMaterial::new(
                    terrain,
                    fog,
                    &outer_visible,
                    &outer_terrain,
                    terrain_buffer,
                )),
                transform: Transform::from_translation(position.0.into()),
                ..default()
            },
            outer_visible,
            outer_terrain: OuterTerrain(outer_terrain),
            ..default()
        }
    }
}

impl Role for ZoneRole {
    fn attach(self, entity: &mut EntityWorldMut) {
        entity.insert((
            self.pickable,
            self.interaction,
            self.not_shadow_caster,
            self.material_mesh_bundle,
            self.outer_visible,
            self.outer_terrain,
        ));
    }
}

#[derive(Bundle, Default)]
pub struct ZoneBundle {
    terrain: TerrainId,
    fog: Fog,
    position: MapPosition,
    zone_decorations: ZoneDecorations,
}

impl ZoneBundle {
    pub fn new(position: HexCoord, prototype: &ZonePrototype) -> Self {
        let terrain = TerrainId(prototype.terrain);
        let mut filliter = prototype.random_fill.iter();
        Self {
            position: MapPosition(position),
            terrain,
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
        outer_terrain: Neighbours<Id<Terrain>>,
    ) -> (Self, ZoneRole) {
        let outer_visible = OuterVisible::default();
        let zone_role = ZoneRole::new(
            zone_params,
            &self.position,
            &self.terrain,
            &self.fog,
            outer_visible,
            outer_terrain,
        );
        (self, zone_role)
    }
}
