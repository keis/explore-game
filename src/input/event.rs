use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct ZoneActivated(pub Entity);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct ZoneOver(pub Entity);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct ZoneOut(pub Entity);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct Select(pub Entity);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct Deselect(pub Entity);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct SelectionOver(pub Entity);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct SelectionOut(pub Entity);
