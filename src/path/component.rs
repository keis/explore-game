use crate::map::HexCoord;
use bevy::prelude::*;
use splines::Spline;
use std::collections::VecDeque;

#[derive(Component)]
pub struct Path {
    pub spline: Spline<f32, Vec3>,
    pub steps: u32,
    pub stroke: f32,
}

#[derive(Component)]
pub struct PathDisplay {
    pub path_guided: Entity,
}

#[derive(Component, Default, Debug)]
pub struct PathGuided {
    pub path: VecDeque<HexCoord>,
    current: Option<HexCoord>,
}

impl PathGuided {
    pub fn path(&mut self, path: Vec<HexCoord>) {
        self.path = VecDeque::from(path);
        self.current = self.path.pop_front();
    }

    pub fn advance(&mut self) {
        self.current = self.path.pop_front();
    }

    pub fn next(&self) -> Option<&HexCoord> {
        self.path.front()
    }

    pub fn last(&self) -> Option<&HexCoord> {
        self.path.back()
    }

    pub fn current(&self) -> Option<HexCoord> {
        self.current
    }
}
