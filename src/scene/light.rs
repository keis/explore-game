use crate::{turn::Period, ExplError};
use bevy::prelude::*;
use bevy_tweening::{Animator, Lens, Targetable, Tracks, Tween};
use std::time::Duration;

// Yoinked from bevy_tweening
trait ColorLerper {
    fn lerp(&self, target: &Self, ratio: f32) -> Self;
}

impl ColorLerper for Color {
    fn lerp(&self, target: &Color, ratio: f32) -> Color {
        let linear = self.to_linear();
        let target = target.to_linear();
        let r = linear.red.lerp(target.red, ratio);
        let g = linear.green.lerp(target.green, ratio);
        let b = linear.blue.lerp(target.blue, ratio);
        let a = linear.alpha.lerp(target.alpha, ratio);
        Color::linear_rgba(r, g, b, a)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LightIlluminanceLens {
    pub start: f32,
    pub end: f32,
}

impl Lens<DirectionalLight> for LightIlluminanceLens {
    fn lerp(&mut self, target: &mut dyn Targetable<DirectionalLight>, ratio: f32) {
        target.illuminance = self.start.lerp(self.end, ratio);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct LightColorLens {
    pub start: Color,
    pub end: Color,
}

impl Lens<DirectionalLight> for LightColorLens {
    fn lerp(&mut self, target: &mut dyn Targetable<DirectionalLight>, ratio: f32) {
        target.color = self.start.lerp(&self.end, ratio);
    }
}

pub fn spawn_light(mut commands: Commands) {
    commands.spawn((
        Name::new("Lucifer"),
        DirectionalLight {
            illuminance: 0.0,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 10.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_3),
            ..default()
        },
        Animator::new(Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_secs(2),
            LightIlluminanceLens {
                start: 0.0,
                end: 11_0000.0,
            },
        )),
    ));
}

pub fn apply_period_light(
    period: Res<Period>,
    mut light_query: Query<(&DirectionalLight, &mut Animator<DirectionalLight>)>,
) -> Result<(), ExplError> {
    let (light, mut animator) = light_query.single_mut()?;
    let (illuminance, color) = match *period {
        Period::Morning => (8_000.0, Color::srgb(1.0, 0.9, 0.9)),
        Period::Day => (11_000.0, Color::srgb(1.0, 1.0, 1.0)),
        Period::Evening => (10_000.0, Color::srgb(1.0, 0.8, 0.8)),
        Period::Night => (6_000.0, Color::srgb(0.8, 0.8, 1.0)),
    };
    let tween = Tracks::new([
        Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(2),
            LightIlluminanceLens {
                start: light.illuminance,
                end: illuminance,
            },
        ),
        Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(2),
            LightColorLens {
                start: light.color,
                end: color,
            },
        ),
    ]);
    animator.set_tweenable(tween);
    Ok(())
}
