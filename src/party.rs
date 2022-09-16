use crate::Turn;

use bevy::ecs::system::Command;
use bevy::prelude::*;
use smallvec::SmallVec;

#[derive(Component, Debug)]
pub struct Party {
    pub name: String,
    pub movement_points: u32,
    pub supplies: u32,
    pub members: SmallVec<[Entity; 8]>,
}

#[derive(Component)]
pub struct PartyMember {
    pub party: Entity,
}

pub struct JoinParty {
    pub party: Entity,
    pub members: SmallVec<[Entity; 8]>,
}

impl Command for JoinParty {
    fn write(mut self, world: &mut World) {
        let mut entity = world.entity_mut(self.party);
        if let Some(mut party) = entity.get_mut::<Party>() {
            party.members.append(&mut self.members);
        }
    }
}

pub fn reset_movement_points(turn: Res<Turn>, mut party_query: Query<&mut Party>) {
    if turn.is_changed() {
        for mut party in party_query.iter_mut() {
            party.movement_points = 2;
        }
    }
}
