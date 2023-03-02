use super::{coord_to_vec3, Fog, GameMap, HexAssets, HexCoord, MapEvent, MapPosition, MapPresence};
use crate::{
    assets::MainAssets,
    camp::Camp,
    crystals::CrystalDeposit,
    material::{TerrainMaterial, ZoneMaterial},
};
use bevy::prelude::*;
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

#[derive(Component)]
pub struct ZoneDecorationCrystals;

#[derive(Component)]
pub struct ZoneDecorationTree;

#[derive(Component, Copy, Clone, Debug, Default)]
pub struct Zone {
    pub terrain: Terrain,
}

impl Zone {
    pub fn is_walkable(&self) -> bool {
        self.terrain != Terrain::Ocean
    }
}

pub struct ZonePrototype {
    pub terrain: Terrain,
    pub random_fill: Vec<(Vec2, f32)>,
    pub crystals: bool,
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
}

fn zone_material(assets: &Res<MainAssets>, terrain: Terrain) -> ZoneMaterial {
    match terrain {
        Terrain::Ocean => ZoneMaterial {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture: Some(assets.ocean_texture.clone()),
            visible: 0,
            explored: 0,
            hover: 0,
        },
        Terrain::Mountain => ZoneMaterial {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture: Some(assets.mountain_texture.clone()),
            visible: 0,
            explored: 0,
            hover: 0,
        },
        Terrain::Forest => ZoneMaterial {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture: Some(assets.forest_texture.clone()),
            visible: 0,
            explored: 0,
            hover: 0,
        },
    }
}

#[allow(clippy::type_complexity)]
pub fn spawn_zone(
    commands: &mut Commands,
    params: &mut ParamSet<(
        Res<MainAssets>,
        Res<HexAssets>,
        ResMut<Assets<ZoneMaterial>>,
        ResMut<Assets<TerrainMaterial>>,
    )>,
    position: HexCoord,
    ZonePrototype {
        terrain,
        random_fill,
        crystals,
    }: &ZonePrototype,
) -> Entity {
    let material = zone_material(&params.p0(), *terrain);
    let zone_entity = commands
        .spawn((
            ZoneBundle {
                position: MapPosition(position),
                zone: Zone { terrain: *terrain },
                ..default()
            },
            MaterialMeshBundle {
                mesh: params.p1().mesh.clone(),
                material: params.p2().add(material),
                transform: Transform::from_translation(coord_to_vec3(position))
                    .with_rotation(Quat::from_rotation_y((90f32).to_radians())),
                ..default()
            },
        ))
        .with_children(|parent| match terrain {
            Terrain::Forest => {
                let mut filliter = random_fill.iter();
                if *crystals {
                    let (pos, scale) = filliter.next().unwrap();
                    parent.spawn((
                        Fog::default(),
                        ZoneDecorationCrystals,
                        MaterialMeshBundle {
                            mesh: params.p0().crystals_mesh.clone(),
                            material: params.p3().add(TerrainMaterial {
                                color: Color::rgba(0.7, 0.4, 0.4, 0.777),
                                visible: 0,
                                explored: 0,
                            }),
                            visibility: Visibility { is_visible: false },
                            transform: Transform::from_translation(Vec3::new(pos.x, 0.0, pos.y))
                                .with_scale(Vec3::splat(scale * 0.3)),
                            ..default()
                        },
                    ));
                }

                for (pos, scale) in filliter {
                    parent.spawn((
                        Fog::default(),
                        ZoneDecorationTree,
                        MaterialMeshBundle {
                            mesh: params.p0().pine_mesh.clone(),
                            material: params.p3().add(TerrainMaterial {
                                color: Color::rgb(0.2, 0.7, 0.3),
                                visible: 0,
                                explored: 0,
                            }),
                            visibility: Visibility { is_visible: false },
                            transform: Transform::from_translation(Vec3::new(pos.x, 0.0, pos.y))
                                .with_scale(Vec3::splat(scale * 0.5)),
                            ..default()
                        },
                    ));
                }
            }
            Terrain::Mountain => {}
            _ => {}
        })
        .id();

    if *crystals {
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
                visibility.is_visible = false;
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
    for event in events.iter() {
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
            visibility.is_visible = true;
        }
    }
}
