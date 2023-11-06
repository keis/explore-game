use super::{asset::HexAssets, component::*};
use crate::{
    assets::MainAssets,
    map::{Fog, HexCoord, MapPosition},
    map_generator::ZonePrototype,
    material::{TerrainMaterial, WaterMaterial, ZoneMaterial},
};
use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::prelude::{Pickable, PickingInteraction};
use glam::Vec3Swizzles;

pub type ZoneDecorationParams<'w> = (Res<'w, MainAssets>, ResMut<'w, Assets<TerrainMaterial>>);

#[derive(Bundle)]
pub struct ZoneDecorationCrystalsBundle {
    fog: Fog,
    zone_decoration_crystals: ZoneDecorationCrystals,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl ZoneDecorationCrystalsBundle {
    pub fn new(
        (main_assets, terrain_materials): &mut ZoneDecorationParams,
        height: &Height,
        fog: &Fog,
        position: HexCoord,
        relative: Vec2,
        scale: f32,
    ) -> Self {
        Self {
            fog: Fog::default(),
            zone_decoration_crystals: ZoneDecorationCrystals,
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.crystals_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgba(0.7, 0.4, 0.4, 0.777),
                    ..default()
                }),
                visibility: if fog.visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                transform: Transform::from_translation(Vec3::new(
                    relative.x,
                    height.height_at(relative, Vec3::from(position).xz() + relative),
                    relative.y,
                ))
                .with_scale(Vec3::splat(scale * 0.3)),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct ZoneDecorationTreeBundle {
    fog: Fog,
    zone_decoration_tree: ZoneDecorationTree,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl ZoneDecorationTreeBundle {
    pub fn new(
        (main_assets, terrain_materials): &mut ZoneDecorationParams,
        height: &Height,
        fog: &Fog,
        position: HexCoord,
        relative: Vec2,
        scale: f32,
    ) -> Self {
        Self {
            fog: *fog,
            zone_decoration_tree: ZoneDecorationTree,
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.pine_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    texture: Some(main_assets.forest_texture.clone()),
                    ..default()
                }),
                visibility: if fog.visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                transform: Transform::from_translation(Vec3::new(
                    relative.x,
                    height.height_at(relative, Vec3::from(position).xz() + relative),
                    relative.y,
                ))
                .with_scale(Vec3::splat(scale * 0.5)),
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

pub type ZoneParams<'w> = (
    Res<'w, MainAssets>,
    Res<'w, HexAssets>,
    ResMut<'w, Assets<ZoneMaterial>>,
);

#[derive(Bundle, Default)]
pub struct ZoneBundle {
    terrain: Terrain,
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
        let terrain = prototype.terrain;
        let height = Height {
            height_amp: prototype.height_amp,
            height_base: prototype.height_base,
            outer_amp: prototype.outer_amp.into(),
            outer_base: prototype.outer_base.into(),
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
                        .map(|(pos, scale)| ZoneDecorationDetail(*pos, *scale))
                } else {
                    None
                },
                tree_details: filliter
                    .map(|(pos, scale)| ZoneDecorationDetail(*pos, *scale))
                    .collect(),
            },
            ..default()
        }
    }

    pub fn with_fluff(self, zone_params: &mut ZoneParams) -> (Self, ZoneFluffBundle) {
        let outer_visible = OuterVisible::default();
        let fluff = ZoneFluffBundle::new(
            zone_params,
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
        (main_assets, hex_assets, zone_materials): &mut ZoneParams,
        position: &MapPosition,
        terrain: &Terrain,
        height: &Height,
        fog: &Fog,
        outer_visible: OuterVisible,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: zone_materials.add(ZoneMaterial::new(
                    main_assets,
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
