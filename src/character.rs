use crate::turn::Turn;
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

pub fn spawn_character(commands: &mut Commands, name: String) -> Entity {
    commands
        .spawn((
            Character { name },
            Movement { points: 2 },
            Selection::default(),
        ))
        .id()
}
