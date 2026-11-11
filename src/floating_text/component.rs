use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component, Default)]
pub struct FloatingText {
    pub(super) progress: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum FloatingTextAlignment {
    Center,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct FloatingTextPrototype {
    pub value: String,
    pub alignment: FloatingTextAlignment,
    pub color: Color,
}

#[derive(Component, Clone, Default, Debug)]
pub struct FloatingTextSource {
    pub(crate) offset: Vec3,
    pub(super) pending: VecDeque<FloatingTextPrototype>,
}

impl FloatingTextSource {
    pub fn with_offset(offset: Vec3) -> Self {
        Self {
            offset,
            ..default()
        }
    }
}

impl FloatingTextSource {
    pub fn add(&mut self, prototype: FloatingTextPrototype) {
        self.pending.push_back(prototype);
    }
}
