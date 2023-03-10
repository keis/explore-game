use crate::{
    combat::{Attack, Health},
    turn::Turn,
};
use bevy::prelude::*;
use bevy_mod_picking::Selection;

#[derive(Component, Debug, Default)]
pub struct Character {
    pub name: String,
}

#[derive(Bundle, Default)]
pub struct CharacterBundle {
    pub character: Character,
    pub movement: Movement,
    pub selection: Selection,
    pub attack: Attack,
    pub health: Health,
}

impl CharacterBundle {
    pub fn new(name: String) -> Self {
        Self {
            character: Character { name },
            movement: Movement { points: 2 },
            attack: Attack(0..8),
            health: Health(10),
            ..default()
        }
    }
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
