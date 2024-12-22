use super::component::*;
use crate::interface::InterfaceAssets;
use bevy::prelude::*;
use bevy_mod_billboard::prelude::*;

#[derive(Bundle, Default)]
pub struct FloatingTextBundle {
    floating_text: FloatingText,
    billboard_text: BillboardText,
    transform: Transform,
    text: Text,
    text_font: TextFont,
    text_color: TextColor,
    text_layout: TextLayout,
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
            transform: Transform::from_translation(
                source
                    + match alignment {
                        FloatingTextAlignment::Center => Vec3::new(-0.0, 0.0, 0.2),
                        FloatingTextAlignment::Left => Vec3::new(-0.1, 0.0, 0.2),
                        FloatingTextAlignment::Right => Vec3::new(0.1, 0.0, 0.2),
                    },
            )
            .with_scale(Vec3::new(0.01, 0.01, 0.01)),
            text: Text::new(value),
            text_font: TextFont {
                font: interface_assets.font.clone(),
                font_size: 26.0,
                ..default()
            },
            text_color: TextColor(color),
            text_layout: TextLayout::new_with_justify(JustifyText::Center),
            ..default()
        }
    }
}
