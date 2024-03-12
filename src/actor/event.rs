use bevy::prelude::*;

#[derive(Event)]
pub enum SlideEvent {
    Stopped,
}

#[derive(Event)]
pub enum GroupEvent {
    MemberAdded { group: Entity, member: Entity },
    MemberRemoved { group: Entity, member: Entity },
}
