use super::{coord_to_vec3, Fog, HexAssets, HexCoord, MapPosition};
use crate::{assets::MainAssets, material::ZoneMaterial};
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

#[derive(Component, Copy, Clone, Debug, Default)]
pub struct Zone {
    pub terrain: Terrain,
}

impl Zone {
    pub fn is_walkable(&self) -> bool {
        self.terrain != Terrain::Ocean
    }
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
            visible: 1,
            explored: 1,
            hover: 0,
        },
        Terrain::Mountain => ZoneMaterial {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture: Some(assets.mountain_texture.clone()),
            visible: 1,
            explored: 1,
            hover: 0,
        },
        Terrain::Forest => ZoneMaterial {
            cloud_texture: Some(assets.cloud_texture.clone()),
            terrain_texture: Some(assets.forest_texture.clone()),
            visible: 1,
            explored: 1,
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
    )>,
    position: HexCoord,
    terrain: Terrain,
) -> Entity {
    let material = zone_material(&params.p0(), terrain);
    commands
        .spawn((
            ZoneBundle {
                position: MapPosition(position),
                zone: Zone { terrain },
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
        .id()
}
