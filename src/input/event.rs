use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct Select(pub Entity);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct Deselect(pub Entity);
