use crate::Turn;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Party {
    pub name: String,
    pub movement_points: u32,
}

pub fn reset_movement_points(turn: Res<Turn>, mut party_query: Query<&mut Party>) {
    if turn.is_changed() {
        for mut party in party_query.iter_mut() {
            party.movement_points = 2;
        }
    }
}
