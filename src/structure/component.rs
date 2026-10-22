use super::asset::Structure;
use crate::{
    actor::Actor, creature::Creature, floating_text::FloatingTextSource, input::Selection,
    inventory::Inventory,
};
use bevy::prelude::*;
use expl_codex::Id;
use expl_map::{Fog, FogRevealer, ViewRadius};

#[derive(Component, Reflect, Default, Deref)]
#[reflect(Component)]
pub struct StructureId(pub Id<Structure>);

impl StructureId {
    pub fn from_tag(tag: &str) -> Self {
        Self(Id::from_tag(tag))
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
#[require(
    Inventory,
    ViewRadius,
    Fog,
    FogRevealer,
    Selection,
    FloatingTextSource,
    Transform,
    Visibility
)]
pub struct Camp {
    pub name: String,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
#[require(Inventory)]
pub struct SafeHaven;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
#[require(Fog, Selection, FloatingTextSource, Transform, Visibility)]
pub struct Portal {
    pub open: bool,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
#[require(Fog, Selection, FloatingTextSource, Transform, Visibility)]
pub struct Spawner {
    pub charge: u8,
    pub creature: Id<Creature>,
    pub actor: Id<Actor>,
}

#[cfg(test)]
mod tests {
    use super::{Camp, Portal, SafeHaven, Spawner};
    use crate::{floating_text::FloatingTextSource, input::Selection, inventory::Inventory};
    use bevy::prelude::*;
    use expl_map::{Fog, FogRevealer, ViewRadius};

    #[test]
    fn markers_require_bundle_companions() {
        let mut app = App::new();

        let camp = app.world_mut().spawn(Camp::default()).id();
        let entity = app.world().entity(camp);
        assert!(entity.contains::<Inventory>());
        assert!(entity.contains::<ViewRadius>());
        assert!(entity.contains::<Fog>());
        assert!(entity.contains::<FogRevealer>());
        assert!(entity.contains::<Selection>());
        assert!(entity.contains::<FloatingTextSource>());
        assert!(entity.contains::<Transform>());
        assert!(entity.contains::<Visibility>());

        let haven = app.world_mut().spawn(SafeHaven).id();
        let entity = app.world().entity(haven);
        assert!(entity.contains::<Inventory>());

        let portal = app.world_mut().spawn(Portal::default()).id();
        let entity = app.world().entity(portal);
        assert!(entity.contains::<Fog>());
        assert!(entity.contains::<Selection>());
        assert!(entity.contains::<FloatingTextSource>());
        assert!(entity.contains::<Transform>());
        assert!(entity.contains::<Visibility>());

        let spawner = app.world_mut().spawn(Spawner::default()).id();
        let entity = app.world().entity(spawner);
        assert!(entity.contains::<Fog>());
        assert!(entity.contains::<Selection>());
        assert!(entity.contains::<FloatingTextSource>());
        assert!(entity.contains::<Transform>());
        assert!(entity.contains::<Visibility>());
    }
}
