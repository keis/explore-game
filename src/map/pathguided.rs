use super::HexCoord;
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component, Default)]
pub struct PathGuided {
    path: VecDeque<HexCoord>,
}

impl PathGuided {
    pub fn path(&mut self, path: Vec<HexCoord>) {
        self.path = VecDeque::from(path);
        self.path.pop_front();
    }

    pub fn take_next(&mut self) -> Option<HexCoord> {
        self.path.pop_front()
    }
}
