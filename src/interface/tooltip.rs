use super::{color::NORMAL, InterfaceAssets};
use bevy::prelude::*;

#[derive(Component)]
pub struct Tooltip;

#[derive(Component)]
pub struct TooltipTarget;

pub fn spawn_tooltip(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    tooltip: impl Into<String>,
) {
    parent
        .spawn((
            Tooltip,
            NodeBundle {
                background_color: NORMAL.into(),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(40.0),
                        ..default()
                    },
                    display: Display::None,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                tooltip,
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            ));
        });
}

#[allow(clippy::type_complexity)]
pub fn show_tooltip_on_hover(
    interaction_query: Query<
        (&Interaction, &Children),
        (With<TooltipTarget>, Changed<Interaction>),
    >,
    mut tooltip_query: Query<&mut Style, With<Tooltip>>,
) {
    for (&interaction, children) in &interaction_query {
        match interaction {
            Interaction::Hovered => {
                for &child in children.iter() {
                    if let Ok(mut tooltip) = tooltip_query.get_mut(child) {
                        tooltip.display = Display::Flex;
                    }
                }
            }
            Interaction::None => {
                for &child in children.iter() {
                    if let Ok(mut tooltip) = tooltip_query.get_mut(child) {
                        tooltip.display = Display::None;
                    }
                }
            }
            _ => {}
        }
    }
}
