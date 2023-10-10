use crate::interface::InterfaceAssets;
use bevy::prelude::*;
use bevy_mod_billboard::prelude::*;
use interpolation::Ease;
use std::collections::VecDeque;

#[derive(Component, Default)]
pub struct FloatingText {
    progress: f32,
}

#[derive(Bundle, Default)]
pub struct FloatingTextBundle {
    floating_text: FloatingText,
    billboard_text_bundle: BillboardTextBundle,
}

impl FloatingTextBundle {
    pub fn new(
        interface_assets: &Res<InterfaceAssets>,
        source: Vec3,
        FloatingTextPrototype {
            value,
            alignment,
            color,
        }: FloatingTextPrototype,
    ) -> Self {
        Self {
            billboard_text_bundle: BillboardTextBundle {
                transform: Transform::from_translation(
                    source
                        + match alignment {
                            FloatingTextAlignment::Center => Vec3::new(-0.0, 0.0, 0.2),
                            FloatingTextAlignment::Left => Vec3::new(-0.1, 0.0, 0.2),
                            FloatingTextAlignment::Right => Vec3::new(0.1, 0.0, 0.2),
                        },
                )
                .with_scale(Vec3::new(0.01, 0.01, 0.01)),
                text: Text::from_sections([TextSection {
                    value,
                    style: TextStyle {
                        font_size: 26.0,
                        font: interface_assets.font.clone(),
                        color,
                    },
                }])
                .with_alignment(TextAlignment::Center),
                ..default()
            },
            ..default()
        }
    }
}

pub enum FloatingTextAlignment {
    Center,
    Left,
    Right,
}

pub struct FloatingTextPrototype {
    pub value: String,
    pub alignment: FloatingTextAlignment,
    pub color: Color,
}

#[derive(Component, Default)]
pub struct FloatingTextSource {
    offset: Vec3,
    pending: VecDeque<FloatingTextPrototype>,
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

pub fn spawn_floating_text(
    mut commands: Commands,
    mut source_query: Query<(&mut FloatingTextSource, &GlobalTransform)>,
    interface_assets: Res<InterfaceAssets>,
) {
    for (mut source, transform) in &mut source_query {
        if let Some(prototype) = source.pending.pop_front() {
            commands.spawn(FloatingTextBundle::new(
                &interface_assets,
                transform.translation() + source.offset,
                prototype,
            ));
        }
    }
}

pub fn float_and_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut floating_text_query: Query<(Entity, &mut FloatingText, &mut Transform, &mut Text)>,
) {
    for (entity, mut floating_text, mut transform, mut text) in &mut floating_text_query {
        let progress = 0.6 * time.delta_seconds();
        floating_text.progress += progress;
        if floating_text.progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y += progress;
        text.sections[0].style.color.set_a(
            1.0 - ((floating_text.progress - 0.5) / 0.5)
                .clamp(0.0, 1.0)
                .quadratic_out(),
        );
    }
}
