use crate::turn::Period;
use bevy::prelude::*;
use bevy_tweening::{Animator, EaseFunction, Lens, Tracks, Tween};
use std::time::Duration;

// Yoinked from bevy_tweening
trait ColorLerper {
    fn lerp(&self, target: &Self, ratio: f32) -> Self;
}

impl ColorLerper for Color {
    fn lerp(&self, target: &Color, ratio: f32) -> Color {
        let r = self.r().lerp(target.r(), ratio);
        let g = self.g().lerp(target.g(), ratio);
        let b = self.b().lerp(target.b(), ratio);
        let a = self.a().lerp(target.a(), ratio);
        Color::rgba(r, g, b, a)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LightIlluminanceLens {
    pub start: f32,
    pub end: f32,
}

impl Lens<DirectionalLight> for LightIlluminanceLens {
    fn lerp(&mut self, target: &mut DirectionalLight, ratio: f32) {
        target.illuminance = self.start.lerp(self.end, ratio);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct LightColorLens {
    pub start: Color,
    pub end: Color,
}

impl Lens<DirectionalLight> for LightColorLens {
    fn lerp(&mut self, target: &mut DirectionalLight, ratio: f32) {
        target.color = self.start.lerp(&self.end, ratio);
    }
}

pub fn spawn_light(mut commands: Commands) {
    commands.spawn((
        Name::new("Lucifer"),
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 0.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 10.0, 0.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_3),
                ..default()
            },
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
) {
    let (light, mut animator) = light_query.single_mut();
    let (illuminance, color) = match *period {
        Period::Morning => (8_000.0, Color::rgb(1.0, 0.9, 0.9)),
        Period::Day => (11_000.0, Color::rgb(1.0, 1.0, 1.0)),
        Period::Evening => (10_000.0, Color::rgb(1.0, 0.8, 0.8)),
        Period::Night => (6_000.0, Color::rgb(0.8, 0.8, 1.0)),
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
}
