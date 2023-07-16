use super::{
    decoration::{
        ZoneDecorationCrystals, ZoneDecorationCrystalsBundle, ZoneDecorationTree,
        ZoneDecorationTreeBundle,
    },
    Fog, Height, HexAssets, HexCoord, MapEvent, MapPosition, MapPresence, MapPrototype,
    PresenceLayer,
};
use crate::{
    assets::MainAssets,
    crystals::CrystalDeposit,
    material::{TerrainMaterial, WaterMaterial, ZoneMaterial},
    structure::Camp,
};
use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::prelude::{Pickable, RaycastPickTarget};
use expl_hexgrid::{layout::SquareGridLayout, Grid};
use glam::Vec3Swizzles;
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

#[derive(Debug)]
pub struct Error(&'static str);

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<char> for Terrain {
    type Error = Error;

    fn try_from(c: char) -> Result<Terrain, Self::Error> {
        match c {
            '%' => Ok(Terrain::Forest),
            '^' => Ok(Terrain::Mountain),
            '~' => Ok(Terrain::Ocean),
            _ => Err(Error("Unknown terrain character")),
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
    pub height_amp: f32,
    pub height_base: f32,
    pub outer_amp: [f32; 6],
    pub outer_base: [f32; 6],
}

#[derive(Bundle, Default)]
pub struct ZoneBundle {
    pub zone: Zone,
    pub height: Height,
    pub fog: Fog,
    pub position: MapPosition,
    pub pickable: Pickable,
    pub raycast_pick_target: RaycastPickTarget,
    pub interaction: Interaction,
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
    let material = zone_material(main_assets, prototype);
    let zone = Zone {
        terrain: prototype.terrain,
    };
    let height = Height {
        height_amp: prototype.height_amp,
        height_base: prototype.height_base,
        outer_amp: prototype.outer_amp,
        outer_base: prototype.outer_base,
    };
    let zone_entity = commands
        .spawn((
            Name::new(format!("Zone {}", position)),
            ZoneBundle {
                position: MapPosition(position),
                zone,
                height,
                ..default()
            },
            MaterialMeshBundle {
                mesh: hex_assets.mesh.clone(),
                material: zone_materials.add(material),
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
                            Vec3::new(
                                pos.x,
                                height.height_at(*pos, Vec3::from(position).xz() + *pos),
                                pos.y,
                            ),
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
                            Vec3::new(
                                pos.x,
                                height.height_at(*pos, Vec3::from(position).xz() + *pos),
                                pos.y,
                            ),
                            *scale,
                        ),
                    ));
                }
            }
            Terrain::Mountain => {}
            Terrain::Ocean => {
                parent.spawn((
                    Name::new("Water"),
                    Fog::default(),
                    NotShadowCaster,
                    MaterialMeshBundle {
                        mesh: hex_assets.mesh.clone(),
                        material: water_materials.add(WaterMaterial {
                            color: Color::rgba(0.1, 0.1, 0.8, 0.4),
                        }),
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
    map_query: Query<&ZoneLayer>,
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
    map_query: Query<(&ZoneLayer, &PresenceLayer)>,
    zone_query: Query<&Children>,
    camp_query: Query<&Camp>,
    mut decoration_query: Query<&mut Visibility, With<ZoneDecorationTree>>,
) {
    let Ok((map, presence_layer)) = map_query.get_single() else { return };
    for event in &mut events {
        let MapEvent::PresenceRemoved { position, .. } = event else { continue };
        if camp_query
            .iter_many(presence_layer.presence(*position))
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
