use super::{color::NORMAL, InterfaceAssets};
use bevy::{color::palettes::css, prelude::*};

#[derive(Component)]
pub struct Tooltip;

#[derive(Component)]
pub struct TooltipTarget;

pub enum TooltipPosition {
    Above,
    Below,
}

pub fn spawn_tooltip(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    position: TooltipPosition,
    tooltip_text: impl Into<String>,
    keybind_text: Option<impl Into<String>>,
) {
    parent
        .spawn((
            Name::new("Tooltip"),
            Tooltip,
            NodeBundle {
                background_color: NORMAL.into(),
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: match position {
                        TooltipPosition::Above => Val::Px(60.0),
                        _ => Val::DEFAULT,
                    },
                    top: match position {
                        TooltipPosition::Below => Val::Px(40.0),
                        _ => Val::DEFAULT,
                    },
                    display: Display::None,
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Tooltip text"),
                TextBundle::from_section(
                    tooltip_text,
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(2.0)),
                    ..default()
                }),
            ));
            if let Some(text) = keybind_text {
                parent.spawn((
                    Name::new("Tooltip keybind"),
                    TextBundle::from_section(
                        text,
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 20.0,
                            color: css::GREEN.into(),
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(2.0)),
                        ..default()
                    }),
                ));
            }
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
