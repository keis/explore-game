use super::queue::GameActionType;
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

#[derive(Component, Copy, Clone, Reflect, Debug, PartialEq)]
#[reflect(Component)]
pub struct CampActionAssignment {
    pub action_type: GameActionType,
}

impl Default for CampActionAssignment {
    fn default() -> Self {
        Self {
            action_type: GameActionType::Move,
        }
    }
}
