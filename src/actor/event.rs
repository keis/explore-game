use bevy::prelude::*;

#[derive(Event)]
pub enum SlideEvent {
    Stopped,
}

#[derive(Event)]
pub struct MemberAdded(pub Entity);

#[derive(Event)]
pub struct MemberRemoved(pub Entity);
