use super::{bundle::*, component::*};
use crate::interface::InterfaceAssets;
use bevy::prelude::*;
use interpolation::Ease;

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
        text.sections[0].style.color.set_alpha(
            1.0 - ((floating_text.progress - 0.5) / 0.5)
                .clamp(0.0, 1.0)
                .quadratic_out(),
        );
    }
}
