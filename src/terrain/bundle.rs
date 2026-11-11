use super::{asset::*, component::*};
use crate::{
    map_generator::ZonePrototype,
    material::{DecorationBuffer, DecorationMaterial, WaterMaterial, ZoneMaterialTemplate},
    role::Role,
    ExplError,
};
use bevy::{
    light::NotShadowCaster,
    prelude::*,
    scene::{bsn, Scene},
};
use expl_codex::{Codex, Id};
use expl_map::{Fog, HexCoord, MapPosition};
use glam::Vec3Swizzles;

pub type ZoneDecorationParams<'w> = (
    Res<'w, DecorationBuffer>,
    ResMut<'w, Assets<DecorationMaterial>>,
);

#[derive(Bundle)]
pub struct ZoneDecorationBundle<Tag: Component> {
    fog: Fog,
    tag: Tag,
    mesh: Mesh3d,
    material: MeshMaterial3d<DecorationMaterial>,
    visibility: Visibility,
    transform: Transform,
}

#[allow(clippy::too_many_arguments)]
impl<Tag: Component> ZoneDecorationBundle<Tag> {
    pub fn new(
        tag: Tag,
        decoration_id: Id<Decoration>,
        (decoration_buffer, decoration_materials): &mut ZoneDecorationParams,
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
            mesh: Mesh3d(decoration.mesh.clone()),
            material: MeshMaterial3d(decoration_materials.add(DecorationMaterial::new(
                &decoration_id,
                fog,
                decoration_buffer,
            ))),
            visibility: Visibility::from(*fog),
            transform: Transform::from_translation(Vec3::new(
                detail.relative.x,
                height.height_at(detail.relative, Vec3::from(position).xz() + detail.relative),
                detail.relative.y,
            ))
            .with_scale(Vec3::splat(detail.scale * decoration.scale)),
        }
    }
}

pub type WaterParams<'w> = (Res<'w, HexAssets>, ResMut<'w, Assets<WaterMaterial>>);

#[derive(Bundle, Default)]
pub struct WaterBundle {
    water: Water,
    fog: Fog,
    not_shadow_caster: NotShadowCaster,
    mesh: Mesh3d,
    material: MeshMaterial3d<WaterMaterial>,
    transform: Transform,
}

impl WaterBundle {
    pub fn new((hex_assets, water_materials): &mut WaterParams) -> Self {
        Self {
            mesh: Mesh3d(hex_assets.mesh.clone()),
            material: MeshMaterial3d(water_materials.add(WaterMaterial {
                color: Color::srgba(0.1, 0.1, 0.8, 0.4),
            })),
            transform: Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)),
            ..default()
        }
    }
}

/// Role presenting a zone.
///
/// Only the [`Id`] is stored; the material is built from the terrain buffer
/// and the zone's own [`Fog`], [`OuterVisible`] and [`OuterTerrain`] when the
/// role is attached.
pub struct ZoneRole {
    terrain_id: Id<Terrain>,
}

impl ZoneRole {
    pub fn new(terrain_id: Id<Terrain>) -> Self {
        Self { terrain_id }
    }
}

impl Role for ZoneRole {
    fn scene(self, world: &World) -> Result<impl Scene, ExplError> {
        let terrain_id = self.terrain_id;
        let mesh = world
            .get_resource::<HexAssets>()
            .ok_or(ExplError::ResourceMissing)?
            .mesh
            .clone();

        Ok(bsn! {
            ~NotShadowCaster
            Mesh3d(mesh)
            ~ZoneMaterialTemplate { input: terrain_id }
        })
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

    pub fn with_fluff(self) -> (Self, ZoneRole) {
        let zone_role = ZoneRole::new(self.terrain.0);
        (self, zone_role)
    }
}
