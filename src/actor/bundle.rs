use super::{asset::*, component::*};
use crate::{
    creature::{Creature, CreatureBundle, Movement},
    input::{DefaultOutlineVolume, Selection, SelectionBundle},
    inventory::Inventory,
    map::{FogRevealer, HexCoord, MapPresence, ViewRadius},
    path::PathGuided,
    role::Role,
    terrain::HeightQuery,
};
use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use expl_codex::{Codex, Id};

pub type ActorParams<'w, 's> = (ResMut<'w, Assets<StandardMaterial>>, HeightQuery<'w, 's>);

#[derive(Default)]
pub struct ActorRole {
    // Insert
    spatial_bundle: SpatialBundle,
    // Child
    pbr_bundle: PbrBundle,
    default_outline_volume: DefaultOutlineVolume,
    outline_bundle: OutlineBundle,
}

impl ActorRole {
    pub fn new(
        (standard_materials, height_query): &mut ActorParams,
        actor_codex: &Codex<Actor>,
        actor_id: Id<Actor>,
        presence: &MapPresence,
    ) -> Self {
        let actor_data = &actor_codex[&actor_id];
        let offset = Vec3::new(0.0, actor_data.offset, 0.0);
        let outline = OutlineVolume {
            visible: true,
            width: 2.0,
            colour: actor_data.outline_color,
        };
        Self {
            spatial_bundle: SpatialBundle {
                transform: Transform::from_translation(
                    height_query.adjust(presence.position.into()),
                ),
                ..default()
            },
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
        }
    }
}

impl Role for ActorRole {
    fn attach(self, entity: &mut EntityWorldMut) {
        entity
            .insert((self.spatial_bundle,))
            .with_children(|parent| {
                parent.spawn((
                    self.pbr_bundle,
                    self.default_outline_volume,
                    self.outline_bundle,
                ));
            });
    }
}

#[derive(Default)]
pub struct PartyRole {
    // Insert
    selection_bundle: SelectionBundle,
    path_guided: PathGuided,
}

impl Role for PartyRole {
    fn attach(self, entity: &mut EntityWorldMut) {
        entity.insert((self.selection_bundle, self.path_guided));
    }
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
pub struct EnemyBundle {
    actor_id: ActorId,
    creature: CreatureBundle,
    enemy: Enemy,
    presence: MapPresence,
    view_radius: ViewRadius,
    slide: Slide,
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
    ) -> (Self, PartyRole, ActorRole) {
        let party_role = PartyRole::default();
        let actor_role = ActorRole::new(party_params, actor_codex, *self.actor_id, &self.presence);
        (self, party_role, actor_role)
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
    ) -> (Self, ActorRole) {
        let actor_role = ActorRole::new(enemy_params, actor_codex, *self.actor_id, &self.presence);
        (self, actor_role)
    }
}
