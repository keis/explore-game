use super::{Fog, HexAssets, HexCoord, MapPosition};
use crate::{
    assets::MainAssets,
    crystals::CrystalDeposit,
    map_generator::{MapPrototype, ZonePrototype},
    material::{TerrainMaterial, WaterMaterial, ZoneMaterial},
    scene::save,
    terrain::{
        Height, Terrain, WaterBundle, ZoneDecorationCrystalsBundle, ZoneDecorationTreeBundle,
    },
};
use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::prelude::{Pickable, PickingInteraction, RaycastPickTarget};
use expl_hexgrid::{layout::SquareGridLayout, Grid};

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

pub fn zone_layer_from_prototype<F>(
    commands: &mut Commands,
    prototype: &MapPrototype,
    mut spawn_tile: F,
) -> ZoneLayer
where
    F: FnMut(&mut Commands, HexCoord, &ZonePrototype) -> Entity,
{
    ZoneLayer {
        tiles: Grid::with_data(
            prototype.tiles.layout,
            prototype
                .tiles
                .iter()
                .map(|(coord, zoneproto)| spawn_tile(commands, coord, zoneproto)),
        ),
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

pub type ZoneParams<'w> = (
    Res<'w, MainAssets>,
    Res<'w, HexAssets>,
    ResMut<'w, Assets<ZoneMaterial>>,
    ResMut<'w, Assets<TerrainMaterial>>,
    ResMut<'w, Assets<WaterMaterial>>,
);

#[allow(clippy::type_complexity)]
pub fn spawn_zone(
    commands: &mut Commands,
    (main_assets, hex_assets, zone_materials, terrain_materials, water_materials): &mut ZoneParams,
    position: HexCoord,
    prototype: &ZonePrototype,
) -> Entity {
    let terrain = prototype.terrain;
    let height = Height {
        height_amp: prototype.height_amp,
        height_base: prototype.height_base,
        outer_amp: prototype.outer_amp.into(),
        outer_base: prototype.outer_base.into(),
    };
    let zone_entity = commands
        .spawn((
            Name::new(format!("Zone {}", position)),
            save::Save,
            ZoneBundle {
                position: MapPosition(position),
                terrain,
                height,
                ..default()
            },
            MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: zone_materials.add(ZoneMaterial::new(main_assets, &terrain, &height)),
                transform: Transform::from_translation(position.into()),
                ..default()
            },
        ))
        .with_children(|parent| match prototype.terrain {
            Terrain::Forest => {
                let mut filliter = prototype.random_fill.iter();
                if prototype.crystals {
                    let (pos, scale) = filliter.next().unwrap();
                    parent.spawn((
                        Name::new("Crystal"),
                        ZoneDecorationCrystalsBundle::new(
                            main_assets,
                            terrain_materials,
                            &height,
                            position,
                            *pos,
                            *scale,
                        ),
                    ));
                }

                for (pos, scale) in filliter {
                    parent.spawn((
                        Name::new("Tree"),
                        ZoneDecorationTreeBundle::new(
                            main_assets,
                            terrain_materials,
                            &height,
                            position,
                            *pos,
                            *scale,
                        ),
                    ));
                }
            }
            Terrain::Mountain => {}
            Terrain::Ocean => {
                parent.spawn((
                    Name::new("Water"),
                    WaterBundle::new(hex_assets, water_materials),
                ));
            }
        })
        .id();

    if prototype.crystals {
        commands
            .entity(zone_entity)
            .insert(CrystalDeposit { amount: 20 });
    }

    zone_entity
}
