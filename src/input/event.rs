use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct ZoneActivated;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct ZoneOver;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct ZoneOut;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct Select;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct Deselect;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct SelectionOver;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Event)]
pub struct SelectionOut;
