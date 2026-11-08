use super::{asset::*, component::*, system_param::actor_codex};
use crate::{
    creature::{Creature, CreatureBundle},
    input::Selection,
    inventory::Inventory,
    path::PathGuided,
    role::Role,
    ExplError,
};
use bevy::{
    ecs::template::{Template, TemplateContext},
    prelude::*,
    scene::{bsn, EntityWorldMutSceneExt},
};
use bevy_mod_outline::OutlineVolume;
use expl_codex::{Codex, Id};
use expl_map::{HexCoord, MapPresence, ViewRadius};

/// Role presenting an actor.
///
/// Only the [`Id`] is stored; the codex entry is resolved when the role is
/// attached.
pub struct ActorRole {
    actor_id: Id<Actor>,
}

impl ActorRole {
    pub fn new(actor_id: Id<Actor>) -> Self {
        Self { actor_id }
    }

    /// Builds the presentation scene owned by the role.
    fn scene(actor: &Actor) -> impl Scene {
        let mesh = actor.mesh.clone();

        bsn! {
            Children [
                Transform {
                    translation: Vec3::new(0.0, actor.offset, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(actor.scale)
                }
                Mesh3d(mesh)
                ~ActorMaterialTemplate { color: { actor.color } }
                OutlineVolume {
                    visible: true,
                    width: 2.0,
                    colour: { actor.outline_color }
                }
            ]
        }
    }
}

impl Role for ActorRole {
    fn attach(self, entity: &mut EntityWorldMut) -> Result<(), ExplError> {
        let actor = actor_codex(entity.world())?[&self.actor_id].clone();
        entity.apply_scene(Self::scene(&actor))?;
        Ok(())
    }
}

/// Scene template building the [`StandardMaterial`] for an actor.
#[derive(Default)]
pub struct ActorMaterialTemplate {
    pub color: Color,
}

impl Template for ActorMaterialTemplate {
    type Output = MeshMaterial3d<StandardMaterial>;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Self::Output> {
        let material = context
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::from(self.color));
        Ok(MeshMaterial3d(material))
    }

    fn clone_template(&self) -> Self {
        Self { color: self.color }
    }
}

#[derive(Default)]
pub struct PartyRole {
    // Insert
    selection: Selection,
    path_guided: PathGuided,
}

impl Role for PartyRole {
    fn attach(self, entity: &mut EntityWorldMut) -> Result<(), ExplError> {
        entity.insert((self.selection, self.path_guided));
        Ok(())
    }
}

#[derive(Bundle, Default)]
pub struct CharacterBundle {
    pub creature: CreatureBundle,
    pub character: Character,
}

#[derive(Bundle, Default)]
pub struct PartyBundle {
    actor_id: ActorId,
    party: Party,
    inventory: Inventory,
    presence: MapPresence,
}

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    actor_id: ActorId,
    creature: CreatureBundle,
    enemy: Enemy,
    presence: MapPresence,
    view_radius: ViewRadius,
}

impl CharacterBundle {
    pub fn new(name: String, creature_codex: &Codex<Creature>) -> Self {
        Self {
            character: Character { name },
            creature: CreatureBundle::new(creature_codex, Id::from_tag("warrior")),
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
        }
    }

    pub fn with_fluff(self) -> (Self, PartyRole, ActorRole) {
        let party_role = PartyRole::default();
        let actor_role = ActorRole::new(*self.actor_id);
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

    pub fn with_fluff(self) -> (Self, ActorRole) {
        let actor_role = ActorRole::new(*self.actor_id);
        (self, actor_role)
    }
}
