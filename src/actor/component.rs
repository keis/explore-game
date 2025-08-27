use super::asset::Actor;
use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::*,
};
use expl_codex::Id;
use smallvec::SmallVec;

#[derive(Component, Reflect, Default, Deref)]
#[reflect(Component)]
pub struct ActorId(pub Id<Actor>);

impl ActorId {
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

#[derive(Component, MapEntities, Reflect, Default, Deref)]
#[reflect(Component, MapEntities)]
pub struct Members(#[entities] pub SmallVec<[Entity; 8]>);

#[derive(Component, MapEntities, Reflect)]
#[reflect(Component, MapEntities)]
pub struct Group(#[entities] pub(super) Entity);

impl Group {
    #[inline(always)]
    pub fn get(&self) -> Entity {
        self.0
    }
}

impl FromWorld for Group {
    #[inline(always)]
    fn from_world(_world: &mut World) -> Self {
        Self(Entity::PLACEHOLDER)
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
