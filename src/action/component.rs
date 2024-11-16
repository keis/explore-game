use crate::ExplError;
use bevy::prelude::*;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct ActionPoints {
    pub current: u16,
    pub reset: u16,
}

impl ActionPoints {
    pub fn new(value: u16) -> Self {
        Self {
            current: value,
            reset: value,
        }
    }

    pub fn reset(&mut self) {
        self.current = self.reset;
    }

    pub fn consume(&mut self) -> Result<(), ExplError> {
        if self.current == 0 {
            Err(ExplError::MoveWithoutMovementPoints)
        } else {
            self.current -= 1;
            Ok(())
        }
    }
}
