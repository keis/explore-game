use crate::map::{HeightQuery, Offset};
use bevy::prelude::*;
use interpolation::Ease;

const SLIDE_SPEED: f32 = 1.7;

#[derive(Component)]
pub struct Slide {
    pub start: Vec3,
    pub end: Vec3,
    pub progress: f32,
}

impl Default for Slide {
    fn default() -> Self {
        Slide {
            start: Vec3::ZERO,
            end: Vec3::ZERO,
            progress: 1.0,
        }
    }
}

pub enum SlideEvent {
    Stopped,
}

pub fn slide(
    mut slide_query: Query<(&mut Transform, &mut Slide, &Offset)>,
    height_query: HeightQuery,
    mut events: EventWriter<SlideEvent>,
    time: Res<Time>,
) {
    for (mut transform, mut slide, offset) in slide_query.iter_mut() {
        if slide.progress == 1.0 {
            continue;
        }
        slide.progress = (slide.progress + time.delta_seconds() * SLIDE_SPEED).clamp(0.0, 1.0);
        let position = slide
            .start
            .lerp(slide.end, slide.progress.quadratic_in_out());
        transform.translation = height_query.adjust(position) + offset.0;
        if slide.progress == 1.0 {
            events.send(SlideEvent::Stopped);
        }
    }
}
