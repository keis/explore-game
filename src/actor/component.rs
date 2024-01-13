use super::asset::Creature;
use crate::ExplError;
use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};
use expl_codex::Id;
use smallvec::SmallVec;

#[derive(Component, Reflect, Default, Deref)]
#[reflect(Component)]
pub struct CreatureId(pub Id<Creature>);

impl CreatureId {
    pub fn from_tag(tag: &str) -> Self {
        Self(Id::from_tag(tag))
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Character {
    pub name: String,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Enemy;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Party {
    pub name: String,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Movement {
    pub current: u16,
    pub reset: u16,
}

impl Movement {
    pub fn reset(&mut self) {
        self.current = self.reset;
    }

    pub fn consume(&mut self) -> Result<(), ExplError> {
        if self.current == 0 {
            Err(ExplError::MoveWithoutMovementPoints)
        } else {
            self.current -= 1;
            Ok(())
        }
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Corpse;

#[derive(Component, Reflect, Default)]
#[reflect(Component, MapEntities)]
pub struct Group {
    pub members: SmallVec<[Entity; 8]>,
}

impl MapEntities for Group {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        for entity in &mut self.members {
            *entity = entity_mapper.get_or_reserve(*entity);
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, MapEntities)]
pub struct GroupMember {
    pub group: Option<Entity>,
}

impl MapEntities for GroupMember {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        if let Some(group) = self.group.as_mut() {
            *group = entity_mapper.get_or_reserve(*group);
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Slide {
    pub start: Vec3,
    pub end: Vec3,
    pub progress: f32,
}

impl Default for Slide {
    fn default() -> Self {
        Slide {
            start: Vec3::ZERO,
            end: Vec3::ZERO,
            progress: 1.0,
        }
    }
}
