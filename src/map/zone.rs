use super::{
    decoration::{
        ZoneDecorationCrystals, ZoneDecorationCrystalsBundle, ZoneDecorationPortalBundle,
        ZoneDecorationTree, ZoneDecorationTreeBundle,
    },
    Fog, GameMap, HexAssets, HexCoord, MapEvent, MapPosition, MapPresence,
};
use crate::{
    assets::MainAssets,
    camp::Camp,
    crystals::CrystalDeposit,
    material::{TerrainMaterial, ZoneMaterial},
};
use bevy::{pbr::NotShadowCaster, prelude::*};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub enum Terrain {
    #[default]
    Ocean,
    Mountain,
    Forest,
}

impl Distribution<Terrain> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Terrain {
        match rng.gen_range(0..=2) {
            0 => Terrain::Ocean,
            1 => Terrain::Mountain,
            2 => Terrain::Forest,
            _ => Terrain::Ocean,
        }
    }
}

impl From<Terrain> for char {
    fn from(terrain: Terrain) -> Self {
        match terrain {
            Terrain::Forest => '%',
            Terrain::Mountain => '^',
            Terrain::Ocean => '~',
        }
    }
}

impl TryFrom<char> for Terrain {
    type Error = &'static str;

    fn try_from(c: char) -> Result<Terrain, Self::Error> {
        match c {
            '%' => Ok(Terrain::Forest),
            '^' => Ok(Terrain::Mountain),
            '~' => Ok(Terrain::Ocean),
            _ => Err("Unknown terrain character"),
        }
    }
}

#[derive(Component, Copy, Clone, Debug, Default)]
pub struct Zone {
    pub terrain: Terrain,
}

impl Zone {
    pub fn is_walkable(&self) -> bool {
        self.terrain != Terrain::Ocean
    }
}

#[derive(Default)]
pub struct ZonePrototype {
    pub terrain: Terrain,
    pub random_fill: Vec<(Vec2, f32)>,
    pub crystals: bool,
    pub portal: bool,
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp: [f32; 6],
    pub outer_base: [f32; 6],
}

#[derive(Bundle, Default)]
pub struct ZoneBundle {
    pub zone: Zone,
    pub fog: Fog,
    pub position: MapPosition,
    pub pickable_mesh: bevy_mod_picking::PickableMesh,
    pub hover: bevy_mod_picking::Hover,
    pub no_deselect: bevy_mod_picking::NoDeselect,
    pub interaction: Interaction,
    pub not_shadow_caster: NotShadowCaster,
}

fn zone_material(assets: &Res<MainAssets>, prototype: &ZonePrototype) -> ZoneMaterial {
    let terrain_texture = match prototype.terrain {
        Terrain::Ocean => Some(assets.ocean_texture.clone()),
        Terrain::Mountain => Some(assets.mountain_texture.clone()),
        Terrain::Forest => Some(assets.grass_texture.clone()),
    };

    ZoneMaterial {
        cloud_texture: Some(assets.cloud_texture.clone()),
        terrain_texture,
        height_amp: prototype.height_amp,
        height_base: prototype.height_base,
        outer_amp: prototype.outer_amp,
        outer_base: prototype.outer_base,
        ..default()
    }
}

pub fn update_outer_visible(
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    changed_zone_query: Query<(&Fog, &MapPosition, &Handle<ZoneMaterial>), Changed<Fog>>,
    zone_query: Query<(&Fog, &Handle<ZoneMaterial>)>,
    map_query: Query<&GameMap>,
) {
    let Ok(map) = map_query.get_single() else { return };
    for (fog, position, handle) in &changed_zone_query {
        let Some(mut material) = zone_materials.get_mut(handle) else { continue };
        if fog.explored {
            material.outer_visible = [true; 6];
            for (idx, coord) in position.0.neighbours().enumerate() {
                let Some((neighbour_fog, neighbour_handle)) = map.get(coord).and_then(|&e| zone_query.get(e).ok()) else { continue };
                if neighbour_fog.explored {
                    continue;
                }
                let Some(mut neighbour_material) = zone_materials.get_mut(neighbour_handle) else { continue };
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
    ResMut<'w, Assets<StandardMaterial>>,
);

#[allow(clippy::type_complexity)]
pub fn spawn_zone(
    commands: &mut Commands,
    (main_assets, hex_assets, zone_materials, terrain_materials, standard_materials): &mut ZoneParams,
    position: HexCoord,
    prototype: &ZonePrototype,
) -> Entity {
    let material = zone_material(main_assets, prototype);
    let zone_entity = commands
        .spawn((
            ZoneBundle {
                position: MapPosition(position),
                zone: Zone {
                    terrain: prototype.terrain,
                },
                ..default()
            },
            MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: zone_materials.add(material),
                transform: Transform::from_translation(position.into())
                    .with_rotation(Quat::from_rotation_y((90f32).to_radians())),
                ..default()
            },
        ))
        .with_children(|parent| match prototype.terrain {
            Terrain::Forest => {
                let mut filliter = prototype.random_fill.iter();
                if prototype.portal {
                    parent.spawn(ZoneDecorationPortalBundle::new(
                        main_assets,
                        terrain_materials,
                    ));
                }
                if prototype.crystals {
                    let (pos, scale) = filliter.next().unwrap();
                    parent.spawn(ZoneDecorationCrystalsBundle::new(
                        main_assets,
                        terrain_materials,
                        *pos,
                        *scale,
                    ));
                }

                for (pos, scale) in filliter {
                    parent.spawn(ZoneDecorationTreeBundle::new(
                        main_assets,
                        terrain_materials,
                        *pos,
                        *scale,
                    ));
                }
            }
            Terrain::Mountain => {}
            Terrain::Ocean => {
                parent.spawn((
                    Fog::default(),
                    NotShadowCaster,
                    MaterialMeshBundle {
                        mesh: hex_assets.mesh.clone(),
                        material: standard_materials.add(Color::rgba(0.1, 0.1, 0.8, 0.4).into()),
                        transform: Transform::from_translation(Vec3::new(0.0, -0.1, 0.0)),
                        ..default()
                    },
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

pub fn despawn_empty_crystal_deposit(
    mut commands: Commands,
    crystal_deposit_query: Query<(&CrystalDeposit, &Children), Changed<CrystalDeposit>>,
    zone_decoration_query: Query<Entity, With<ZoneDecorationCrystals>>,
) {
    for (_, children) in crystal_deposit_query
        .iter()
        .filter(|(deposit, _)| deposit.amount == 0)
    {
        for decoration_entity in zone_decoration_query.iter_many(children.iter()) {
            commands.entity(decoration_entity).despawn();
        }
    }
}

pub fn hide_decorations_behind_camp(
    presence_query: Query<&MapPresence, (Changed<MapPresence>, With<Camp>)>,
    map_query: Query<&GameMap>,
    zone_query: Query<&Children>,
    mut decoration_query: Query<(&mut Visibility, &Transform), With<ZoneDecorationTree>>,
) {
    let Ok(map) = map_query.get_single() else { return };
    for presence in &presence_query {
        let Some(children) = map.get(presence.position).and_then(|&e| zone_query.get(e).ok()) else { continue };
        let mut decoration_iter = decoration_query.iter_many_mut(children);
        while let Some((mut visibility, transform)) = decoration_iter.fetch_next() {
            if transform.translation.distance(Vec3::ZERO) < 0.3 {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn show_decorations_behind_camp(
    mut events: EventReader<MapEvent>,
    map_query: Query<&GameMap>,
    zone_query: Query<&Children>,
    camp_query: Query<&Camp>,
    mut decoration_query: Query<&mut Visibility, With<ZoneDecorationTree>>,
) {
    let Ok(map) = map_query.get_single() else { return };
    for event in &mut events {
        let MapEvent::PresenceRemoved { position, .. } = event else { continue };
        if camp_query
            .iter_many(map.presence(*position))
            .next()
            .is_some()
        {
            continue;
        }
        let Some(children) = map.get(*position).and_then(|&e| zone_query.get(e).ok()) else { continue };
        let mut decoration_iter = decoration_query.iter_many_mut(children);
        while let Some(mut visibility) = decoration_iter.fetch_next() {
            *visibility = Visibility::Inherited;
        }
    }
}
