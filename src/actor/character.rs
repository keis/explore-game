use crate::{
    combat::{Attack, Health},
    input::Selection,
};
use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
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
            attack: Attack { low: 0, high: 8 },
            health: Health(10, 10),
            ..default()
        }
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Movement {
    pub points: u32,
}

pub fn reset_movement_points(mut movement_query: Query<&mut Movement>) {
    for mut movement in movement_query.iter_mut() {
        movement.points = 2;
    }
}
