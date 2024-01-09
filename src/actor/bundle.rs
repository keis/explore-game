use super::component::*;
use crate::{
    assets::MainAssets,
    combat::{Attack, Health},
    input::{Selection, SelectionBundle},
    inventory::Inventory,
    map::{FogRevealer, HexCoord, MapPresence, ViewRadius},
    path::PathGuided,
    terrain::HeightQuery,
};
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_mod_picking::prelude::PickHighlight;

#[derive(Bundle, Default)]
pub struct CharacterBundle {
    pub character: Character,
    pub movement: Movement,
    pub selection: Selection,
    pub attack: Attack,
    pub health: Health,
}

impl CharacterBundle {
    pub fn new(name: String) -> Self {
        Self {
            character: Character { name },
            movement: Movement { points: 2 },
            attack: Attack { low: 0, high: 8 },
            health: Health(10, 10),
            ..default()
        }
    }
}

pub type PartyParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<StandardMaterial>>,
    HeightQuery<'w, 's>,
);

#[derive(Bundle, Default)]
pub struct PartyBundle {
    party: Party,
    inventory: Inventory,
    presence: MapPresence,
    group: Group,
    movement: Movement,
    view_radius: ViewRadius,
    slide: Slide,
    fog_revealer: FogRevealer,
}

#[derive(Bundle, Default)]
pub struct PartyFluffBundle {
    selection_bundle: SelectionBundle,
    path_guided: PathGuided,
    spatial_bundle: SpatialBundle,
}

#[derive(Bundle)]
pub struct PartyChildBundle {
    pbr_bundle: PbrBundle,
    pick_highlight: PickHighlight,
    outline_bundle: OutlineBundle,
}

impl PartyBundle {
    pub fn new(position: HexCoord, name: String, supplies: u32) -> Self {
        let presence = MapPresence { position };
        let mut inventory = Inventory::default();
        inventory.add_item(Inventory::SUPPLY, supplies);
        Self {
            party: Party { name },
            inventory,
            presence,
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        party_params: &mut PartyParams,
    ) -> ((Self, PartyFluffBundle), PartyChildBundle) {
        let (fluff, child) = PartyFluffBundle::new(party_params, &self.presence);
        ((self, fluff), child)
    }
}

impl PartyFluffBundle {
    pub fn new(
        (main_assets, standard_materials, height_query): &mut PartyParams,
        presence: &MapPresence,
    ) -> (Self, PartyChildBundle) {
        let offset = Vec3::new(0.0, 0.5, 0.0);
        (
            Self {
                spatial_bundle: SpatialBundle {
                    transform: Transform::from_translation(
                        height_query.adjust(presence.position.into()),
                    ),
                    ..default()
                },
                ..default()
            },
            PartyChildBundle {
                pbr_bundle: PbrBundle {
                    mesh: main_assets.shield_mesh.clone(),
                    material: standard_materials.add(Color::rgb(0.165, 0.631, 0.596).into()),
                    transform: Transform::from_translation(offset).with_scale(Vec3::splat(0.1)),
                    ..default()
                },
                pick_highlight: PickHighlight,
                outline_bundle: OutlineBundle {
                    outline: OutlineVolume {
                        visible: true,
                        width: 2.0,
                        colour: Color::rgb(0.155, 0.621, 0.586),
                    },
                    ..default()
                },
            },
        )
    }
}

pub type EnemyParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<StandardMaterial>>,
    HeightQuery<'w, 's>,
);

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    enemy: Enemy,
    presence: MapPresence,
    view_radius: ViewRadius,
    slide: Slide,
    attack: Attack,
    health: Health,
}

#[derive(Bundle, Default)]
pub struct EnemyFluffBundle {
    spatial_bundle: SpatialBundle,
}

#[derive(Bundle, Default)]
pub struct EnemyChildBundle {
    pbr_bundle: PbrBundle,
    outline_bundle: OutlineBundle,
}

impl EnemyBundle {
    pub fn new(position: HexCoord) -> Self {
        let presence = MapPresence { position };
        Self {
            presence,
            view_radius: ViewRadius(3),
            attack: Attack { low: 1, high: 10 },
            health: Health(20, 20),
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        enemy_params: &mut EnemyParams,
    ) -> ((Self, EnemyFluffBundle), EnemyChildBundle) {
        let (fluff, child) = EnemyFluffBundle::new(enemy_params, &self.presence);
        ((self, fluff), child)
    }
}

impl EnemyFluffBundle {
    pub fn new(
        (main_assets, standard_materials, height_query): &mut EnemyParams,
        presence: &MapPresence,
    ) -> (Self, EnemyChildBundle) {
        let offset = Vec3::new(0.0, 0.05, 0.0);
        (
            Self {
                spatial_bundle: SpatialBundle {
                    transform: Transform::from_translation(
                        height_query.adjust(presence.position.into()),
                    ),
                    visibility: Visibility::Hidden,
                    ..default()
                },
            },
            EnemyChildBundle {
                pbr_bundle: PbrBundle {
                    mesh: main_assets.blob_mesh.clone(),
                    material: standard_materials
                        .add(Color::rgba(0.749, 0.584, 0.901, 0.666).into()),
                    transform: Transform::from_translation(offset).with_scale(Vec3::splat(0.5)),
                    ..default()
                },
                outline_bundle: OutlineBundle {
                    outline: OutlineVolume {
                        visible: true,
                        width: 2.0,
                        colour: Color::rgb(0.739, 0.574, 0.891),
                    },
                    ..default()
                },
            },
        )
    }
}
