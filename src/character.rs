use crate::turn::Turn;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Character {
    pub name: String,
}

#[derive(Component, Default, Debug)]
pub struct Movement {
    pub points: u32,
}

pub fn reset_movement_points(turn: Res<Turn>, mut movement_query: Query<&mut Movement>) {
    if turn.is_changed() {
        for mut movement in movement_query.iter_mut() {
            movement.points = 2;
        }
    }
}
