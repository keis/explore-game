use crate::{
    combat::{Attack, Health},
    input::Selection,
};
use bevy::prelude::*;

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
            health: Health(10, 10),
            ..default()
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct Movement {
    pub points: u32,
}

pub fn reset_movement_points(mut movement_query: Query<&mut Movement>) {
    for mut movement in movement_query.iter_mut() {
        movement.points = 2;
    }
}
