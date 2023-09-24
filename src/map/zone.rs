use super::{Fog, HexAssets, HexCoord, MapPosition};
use crate::{
    assets::MainAssets,
    map_generator::ZonePrototype,
    material::ZoneMaterial,
    terrain::{Height, Terrain, ZoneDecorationDetail, ZoneDecorations},
};
use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::prelude::{Pickable, PickingInteraction, RaycastPickTarget};
use expl_hexgrid::{layout::SquareGridLayout, Grid};

pub type ZoneParams<'w> = (
    Res<'w, MainAssets>,
    Res<'w, HexAssets>,
    ResMut<'w, Assets<ZoneMaterial>>,
);

#[derive(Bundle, Default)]
pub struct ZoneBundle {
    pub terrain: Terrain,
    pub height: Height,
    pub fog: Fog,
    pub position: MapPosition,
    pub pickable: Pickable,
    pub raycast_pick_target: RaycastPickTarget,
    pub interaction: PickingInteraction,
    pub not_shadow_caster: NotShadowCaster,
    pub material_mesh_bundle: MaterialMeshBundle<ZoneMaterial>,
    pub zone_decorations: ZoneDecorations,
}

impl ZoneBundle {
    pub fn new(
        (main_assets, hex_assets, zone_materials): &mut ZoneParams,
        position: HexCoord,
        prototype: &ZonePrototype,
    ) -> Self {
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
            material_mesh_bundle: MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: zone_materials.add(ZoneMaterial::new(main_assets, &terrain, &height)),
                transform: Transform::from_translation(position.into()),
                ..default()
            },
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
}

#[derive(Component)]
pub struct ZoneLayer {
    tiles: Grid<SquareGridLayout, Entity>,
}

impl ZoneLayer {
    pub fn new(layout: SquareGridLayout, tiles: Vec<Entity>) -> Self {
        ZoneLayer {
            tiles: Grid::with_data(layout, tiles),
        }
    }

    pub fn layout(&self) -> SquareGridLayout {
        self.tiles.layout
    }

    pub fn set(&mut self, position: HexCoord, entity: Entity) {
        self.tiles.set(position, entity)
    }

    pub fn get(&self, position: HexCoord) -> Option<&Entity> {
        self.tiles.get(position)
    }
}

pub fn update_outer_visible(
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    changed_zone_query: Query<(&Fog, &MapPosition, &Handle<ZoneMaterial>), Changed<Fog>>,
    zone_query: Query<(&Fog, &Handle<ZoneMaterial>)>,
    map_query: Query<&ZoneLayer>,
) {
    let Ok(map) = map_query.get_single() else { return };
    for (fog, position, handle) in &changed_zone_query {
        if fog.explored {
            let Some(material) = zone_materials.get_mut(handle) else { continue };
            material.outer_visible = [true; 6];
            for (idx, coord) in position.0.neighbours().enumerate() {
                let Some((neighbour_fog, neighbour_handle)) = map.get(coord).and_then(|&e| zone_query.get(e).ok()) else { continue };
                if neighbour_fog.explored {
                    continue;
                }
                let Some(neighbour_material) = zone_materials.get_mut(neighbour_handle) else { continue };
                neighbour_material.outer_visible[(idx + 2) % 6] = true;
                neighbour_material.outer_visible[(idx + 3) % 6] = true;
            }
        }
    }
}
