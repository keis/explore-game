use super::InterfaceAssets;
use bevy::prelude::*;

pub fn spawn_stat_display(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    tag: impl Component,
    image: Handle<Image>,
    value: impl Into<String>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(2.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                    ..default()
                },
                image: image.into(),
                ..default()
            });
            parent.spawn((
                tag,
                TextBundle::from_sections([TextSection::new(
                    value,
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                )]),
            ));
        });
}
