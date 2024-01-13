use super::{asset::*, component::*};
use crate::{
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
use expl_codex::{Codex, Id};

pub type CreatureParams<'w, 's> = (ResMut<'w, Assets<StandardMaterial>>, HeightQuery<'w, 's>);

#[derive(Bundle, Default)]
pub struct CreatureBundle {
    pub creature_id: CreatureId,
    pub movement: Movement,
    pub attack: Attack,
    pub health: Health,
}

#[derive(Bundle, Default)]
pub struct CreatureFluffBundle {
    spatial_bundle: SpatialBundle,
}

#[derive(Bundle)]
pub struct CreatureChildBundle {
    pbr_bundle: PbrBundle,
    pick_highlight: PickHighlight,
    outline_bundle: OutlineBundle,
}

#[derive(Bundle, Default)]
pub struct CharacterBundle {
    pub creature: CreatureBundle,
    pub character: Character,
    pub selection: Selection,
}

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
    creature_fluff: CreatureFluffBundle,
}

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    creature: CreatureBundle,
    enemy: Enemy,
    presence: MapPresence,
    view_radius: ViewRadius,
    slide: Slide,
}

impl CreatureBundle {
    pub fn new(creature_codex: &Codex<Creature>, creature_id: Id<Creature>) -> Self {
        let creature_data = &creature_codex[&creature_id];
        Self {
            creature_id: CreatureId(creature_id),
            movement: Movement {
                current: creature_data.movement,
                reset: creature_data.movement,
            },
            attack: creature_data.attack.clone(),
            health: Health {
                current: creature_data.health,
                max: creature_data.health,
            },
        }
    }
}

impl CreatureFluffBundle {
    pub fn new(
        (standard_materials, height_query): &mut CreatureParams,
        creature_codex: &Codex<Creature>,
        creature_id: Id<Creature>,
        presence: &MapPresence,
    ) -> (Self, CreatureChildBundle) {
        let creature_data = &creature_codex[&creature_id];
        let offset = Vec3::new(0.0, creature_data.offset, 0.0);
        (
            Self {
                spatial_bundle: SpatialBundle {
                    transform: Transform::from_translation(
                        height_query.adjust(presence.position.into()),
                    ),
                    ..default()
                },
            },
            CreatureChildBundle {
                pbr_bundle: PbrBundle {
                    mesh: creature_data.mesh.clone(),
                    material: standard_materials.add(creature_data.color.into()),
                    transform: Transform::from_translation(offset)
                        .with_scale(Vec3::splat(creature_data.scale)),
                    ..default()
                },
                pick_highlight: PickHighlight,
                outline_bundle: OutlineBundle {
                    outline: OutlineVolume {
                        visible: true,
                        width: 2.0,
                        colour: creature_data.outline_color,
                    },
                    ..default()
                },
            },
        )
    }
}

impl CharacterBundle {
    pub fn new(name: String, creature_codex: &Codex<Creature>) -> Self {
        Self {
            character: Character { name },
            creature: CreatureBundle::new(creature_codex, Id::from_tag("warrior")),
            ..default()
        }
    }
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
        party_params: &mut CreatureParams,
        creature_codex: &Codex<Creature>,
    ) -> ((Self, PartyFluffBundle), CreatureChildBundle) {
        let creature_id = Id::from_tag("warrior");
        let (fluff, child) =
            PartyFluffBundle::new(party_params, creature_codex, creature_id, &self.presence);
        ((self, fluff), child)
    }
}

impl PartyFluffBundle {
    pub fn new(
        party_params: &mut CreatureParams,
        creature_codex: &Codex<Creature>,
        creature_id: Id<Creature>,
        presence: &MapPresence,
    ) -> (Self, CreatureChildBundle) {
        let (creature_fluff, child) =
            CreatureFluffBundle::new(party_params, creature_codex, creature_id, presence);
        (
            Self {
                creature_fluff,
                ..default()
            },
            child,
        )
    }
}

impl EnemyBundle {
    pub fn new(
        position: HexCoord,
        creature_codex: &Codex<Creature>,
        creature_id: Id<Creature>,
    ) -> Self {
        let presence = MapPresence { position };
        let view_radius = ViewRadius(creature_codex[&creature_id].view_radius.into());
        Self {
            presence,
            view_radius,
            creature: CreatureBundle::new(creature_codex, creature_id),
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        enemy_params: &mut CreatureParams,
        creature_codex: &Codex<Creature>,
    ) -> ((Self, CreatureFluffBundle), CreatureChildBundle) {
        let (fluff, child) = CreatureFluffBundle::new(
            enemy_params,
            creature_codex,
            *self.creature.creature_id,
            &self.presence,
        );
        ((self, fluff), child)
    }
}
