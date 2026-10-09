use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, EntityEvent)]
pub struct ZoneActivated {
    pub entity: Entity,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EntityEvent)]
pub struct ZoneOver {
    pub entity: Entity,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EntityEvent)]
pub struct ZoneOut {
    pub entity: Entity,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EntityEvent)]
pub struct Select {
    pub entity: Entity,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EntityEvent)]
pub struct Deselect {
    pub entity: Entity,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EntityEvent)]
pub struct SelectionOver {
    pub entity: Entity,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, EntityEvent)]
pub struct SelectionOut {
    pub entity: Entity,
}
