use super::{asset::*, component::*};
use crate::{
    creature::{Creature, CreatureBundle, Movement},
    input::{DefaultOutlineVolume, Selection, SelectionBundle},
    inventory::Inventory,
    map::{FogRevealer, HexCoord, MapPresence, ViewRadius},
    path::PathGuided,
    terrain::HeightQuery,
};
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use expl_codex::{Codex, Id};

pub type ActorParams<'w, 's> = (ResMut<'w, Assets<StandardMaterial>>, HeightQuery<'w, 's>);

#[derive(Bundle, Default)]
pub struct ActorFluffBundle {
    spatial_bundle: SpatialBundle,
}

#[derive(Bundle)]
pub struct ActorChildBundle {
    pbr_bundle: PbrBundle,
    default_outline_volume: DefaultOutlineVolume,
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
    actor_id: ActorId,
    party: Party,
    inventory: Inventory,
    presence: MapPresence,
    members: Members,
    movement: Movement,
    view_radius: ViewRadius,
    slide: Slide,
    fog_revealer: FogRevealer,
}

#[derive(Bundle, Default)]
pub struct PartyFluffBundle {
    selection_bundle: SelectionBundle,
    path_guided: PathGuided,
}

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    actor_id: ActorId,
    creature: CreatureBundle,
    enemy: Enemy,
    presence: MapPresence,
    view_radius: ViewRadius,
    slide: Slide,
}

impl ActorFluffBundle {
    pub fn new(
        (standard_materials, height_query): &mut ActorParams,
        actor_codex: &Codex<Actor>,
        actor_id: Id<Actor>,
        presence: &MapPresence,
    ) -> (Self, ActorChildBundle) {
        let actor_data = &actor_codex[&actor_id];
        let offset = Vec3::new(0.0, actor_data.offset, 0.0);
        let outline = OutlineVolume {
            visible: true,
            width: 2.0,
            colour: actor_data.outline_color,
        };
        (
            Self {
                spatial_bundle: SpatialBundle {
                    transform: Transform::from_translation(
                        height_query.adjust(presence.position.into()),
                    ),
                    ..default()
                },
            },
            ActorChildBundle {
                pbr_bundle: PbrBundle {
                    mesh: actor_data.mesh.clone(),
                    material: standard_materials.add(actor_data.color),
                    transform: Transform::from_translation(offset)
                        .with_scale(Vec3::splat(actor_data.scale)),
                    ..default()
                },
                default_outline_volume: DefaultOutlineVolume(outline.clone()),
                outline_bundle: OutlineBundle {
                    outline,
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
        let actor_id = ActorId::from_tag("party");
        Self {
            party: Party { name },
            actor_id,
            inventory,
            presence,
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        party_params: &mut ActorParams,
        actor_codex: &Codex<Actor>,
    ) -> ((Self, PartyFluffBundle, ActorFluffBundle), ActorChildBundle) {
        let party_fluff = PartyFluffBundle::default();
        let (actor_fluff, child) =
            ActorFluffBundle::new(party_params, actor_codex, *self.actor_id, &self.presence);
        ((self, party_fluff, actor_fluff), child)
    }
}

impl EnemyBundle {
    pub fn new(
        position: HexCoord,
        creature_codex: &Codex<Creature>,
        creature_id: Id<Creature>,
        actor_id: Id<Actor>,
    ) -> Self {
        let presence = MapPresence { position };
        let view_radius = ViewRadius(creature_codex[&creature_id].view_radius.into());
        Self {
            presence,
            view_radius,
            actor_id: ActorId(actor_id),
            creature: CreatureBundle::new(creature_codex, creature_id),
            ..default()
        }
    }

    pub fn with_fluff(
        self,
        enemy_params: &mut ActorParams,
        actor_codex: &Codex<Actor>,
    ) -> ((Self, ActorFluffBundle), ActorChildBundle) {
        let (fluff, child) =
            ActorFluffBundle::new(enemy_params, actor_codex, *self.actor_id, &self.presence);
        ((self, fluff), child)
    }
}
