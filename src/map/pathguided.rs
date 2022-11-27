use super::HexCoord;
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component, Default, Debug)]
pub struct PathGuided {
    pub path: VecDeque<HexCoord>,
}

impl PathGuided {
    pub fn path(&mut self, path: Vec<HexCoord>) {
        self.path = VecDeque::from(path);
        self.path.pop_front();
    }

    pub fn advance(&mut self) {
        self.path.pop_front();
    }

    pub fn next(&self) -> Option<&HexCoord> {
        self.path.front()
    }
}
