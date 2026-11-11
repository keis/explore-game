use super::{asset::*, component::*};
use crate::{
    codex::codex,
    creature::{Creature, CreatureBundle},
    inventory::Inventory,
    material::{MaterialFromTemplate, MaterialTemplate},
    role::Role,
    ExplError,
};
use bevy::{
    ecs::template::TemplateContext,
    prelude::*,
    scene::{bsn, Scene},
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
}

impl Role for ActorRole {
    fn scene(self, world: &World) -> Result<impl Scene, ExplError> {
        let actor_id = self.actor_id;
        let actor = codex::<Actor>(world)?[&actor_id].clone();
        let mesh = actor.mesh.clone();

        Ok(bsn! {
            Children [
                Transform {
                    translation: Vec3::new(0.0, actor.offset, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(actor.scale)
                }
                Mesh3d(mesh)
                ~ActorMaterialTemplate { input: actor_id }
                OutlineVolume {
                    visible: true,
                    width: 2.0,
                    colour: { actor.outline_color }
                }
            ]
        })
    }
}

/// Scene template building the [`StandardMaterial`] for an actor.
pub type ActorMaterialTemplate = MaterialTemplate<StandardMaterial>;

impl MaterialFromTemplate for StandardMaterial {
    type Input = Id<Actor>;

    fn from_template(actor_id: Self::Input, context: &mut TemplateContext) -> Result<Self> {
        let actor = &codex::<Actor>(context.entity.world())?[&actor_id];
        Ok(Self::from(actor.color))
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

    pub fn with_fluff(self) -> (Self, ActorRole) {
        let actor_role = ActorRole::new(*self.actor_id);
        (self, actor_role)
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
