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
    pub path: VecDeque<Entity>,
    current: Option<Entity>,
}

impl PathGuided {
    pub fn path<Path>(&mut self, path: Path)
    where
        Path: IntoIterator<Item = Entity>,
    {
        self.path = VecDeque::from_iter(path);
        self.current = self.path.pop_front();
    }

    pub fn advance(&mut self) {
        self.current = self.path.pop_front();
    }

    pub fn next(&self) -> Option<&Entity> {
        self.path.front()
    }

    pub fn last(&self) -> Option<&Entity> {
        self.path.back()
    }

    pub fn current(&self) -> Option<Entity> {
        self.current
    }
}
