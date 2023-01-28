use crate::{
    indicator::Indicator,
    map::{Offset, PathGuided, ViewRadius},
    slide::Slide,
    turn::Turn,
};

use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use smallvec::SmallVec;

#[derive(Component, Debug, Default)]
pub struct Party {
    pub name: String,
    pub movement_points: u32,
    pub supplies: u32,
    pub members: SmallVec<[Entity; 8]>,
}

#[derive(Bundle, Default)]
pub struct PartyBundle {
    pub party: Party,
    pub pickable_bundle: PickableBundle,
    pub indicator: Indicator,
    pub offset: Offset,
    pub view_radius: ViewRadius,
    pub path_guided: PathGuided,
    pub slide: Slide,
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
        let mut party_entity = world.entity_mut(self.party);
        if let Some(mut party) = party_entity.get_mut::<Party>() {
            party.members.append(&mut self.members);
        }

        for member in self.members {
            let mut member_entity = world.entity_mut(member);
            if let Some(mut party_member) = member_entity.get_mut::<PartyMember>() {
                party_member.party = self.party;
            } else {
                member_entity.insert(PartyMember { party: self.party });
            }
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
