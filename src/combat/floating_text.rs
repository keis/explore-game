use bevy::prelude::*;
use interpolation::Ease;

#[derive(Component, Default)]
pub struct FloatingText {
    progress: f32,
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
