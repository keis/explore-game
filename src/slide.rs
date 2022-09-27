use bevy::prelude::*;

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
    mut slide_query: Query<(&mut Transform, &mut Slide)>,
    mut events: EventWriter<SlideEvent>,
    time: Res<Time>,
) {
    for (mut transform, mut slide) in slide_query.iter_mut() {
        if slide.progress == 1.0 {
            continue;
        }
        slide.progress = (slide.progress + time.delta_seconds()).clamp(0.0, 1.0);
        transform.translation = slide.start.lerp(slide.end, slide.progress);
        if slide.progress == 1.0 {
            events.send(SlideEvent::Stopped);
        }
    }
}
