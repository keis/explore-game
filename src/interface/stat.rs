use super::{style::*, styles::style_small_icon, InterfaceAssets};
use bevy::prelude::*;
use expl_databinding::DataBindingExt;

fn style_stat_display(style: &mut StyleBuilder) {
    style.padding(Val::Px(2.0)).align_items(AlignItems::Center);
}

pub fn spawn_stat_display(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    entity: Entity,
    tag: impl Component,
    image: Handle<Image>,
    value: impl Into<String>,
) {
    parent
        .spawn(NodeBundle::default())
        .with_style(style_stat_display)
        .with_children(|parent| {
            parent
                .spawn(ImageBundle {
                    image: image.into(),
                    ..default()
                })
                .with_style(style_small_icon);
            parent
                .spawn((
                    tag,
                    TextBundle::from_sections([TextSection::new(
                        value,
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    )]),
                ))
                .bind_to(entity);
        });
}
