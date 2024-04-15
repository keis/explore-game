use super::component::*;
use crate::interface::InterfaceAssets;
use bevy::prelude::*;
use bevy_mod_billboard::prelude::*;

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
                .with_justify(JustifyText::Center),
                ..default()
            },
            ..default()
        }
    }
}
