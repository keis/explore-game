use super::color::*;
use crate::ExplError;
use bevy::{hierarchy::HierarchyEvent, prelude::*};

#[derive(Component)]
pub(super) struct TabView;

#[derive(Component)]
pub(super) struct TabViewHeader;

#[derive(Component)]
pub(super) struct TabViewHeaderButton {
    content: Entity,
}

#[derive(Component)]
pub(super) struct TabViewContent {
    pub icon: Handle<Image>,
}

pub(super) fn update_tab_view(
    mut commands: Commands,
    mut hierarchy_events: EventReader<HierarchyEvent>,
    mut style_query: Query<&mut Style>,
    mut header_button_query: Query<(Entity, &TabViewHeaderButton, &mut BackgroundColor)>,
    content_query: Query<&TabViewContent>,
    view_query: Query<&Children, With<TabView>>,
    header_query: Query<&Children, With<TabViewHeader>>,
) {
    for event in hierarchy_events.read() {
        match event {
            HierarchyEvent::ChildAdded { child, parent } => {
                let Ok(children) = view_query.get(*parent) else {
                    continue;
                };
                let Ok(content) = content_query.get(*child) else {
                    continue;
                };
                let header = children[0];
                let has_visible_content = style_query
                    .iter_many(children[1..].iter().filter(|e| **e != *child))
                    .any(|style| style.display != Display::None);
                if has_visible_content {
                    match style_query.get_mut(*child) {
                        Ok(mut style) => style.display = Display::None,
                        Err(err) => warn!("{}", err),
                    };
                }
                commands.get_or_spawn(header).with_children(|parent| {
                    spawn_header_icon(parent, *child, content, !has_visible_content);
                });
            }
            HierarchyEvent::ChildRemoved { child, parent } => {
                let Ok(children) = view_query.get(*parent) else {
                    continue;
                };
                let header = children[0];
                let Ok(header_children) = header_query.get(header) else {
                    continue;
                };
                let has_visible_content = style_query
                    .iter_many(children[1..].iter())
                    .any(|style| style.display != Display::None);
                let mut new_visible_content = None;
                if !has_visible_content && children.len() >= 2 {
                    new_visible_content = Some(children[1]);
                    match style_query.get_mut(children[1]) {
                        Ok(mut style) => style.display = Display::Flex,
                        Err(err) => warn!("{}", err),
                    };
                }
                let mut header_button_iter = header_button_query.iter_many_mut(header_children);
                while let Some((button_entity, header_button, mut background_color)) =
                    header_button_iter.fetch_next()
                {
                    if Some(header_button.content) == new_visible_content {
                        *background_color = SELECTED.into();
                    }
                    if header_button.content == *child {
                        commands.entity(button_entity).remove_parent();
                        commands.entity(button_entity).despawn_recursive();
                    }
                }
            }
            HierarchyEvent::ChildMoved { .. } => {}
        }
    }
}

pub(super) fn handle_tab_view_header_button_interaction(
    mut content_query: Query<(Entity, &mut Style), With<TabViewContent>>,
    mut header_button_query: Query<(&TabViewHeaderButton, &mut BackgroundColor)>,
    interaction_query: Query<(&Interaction, &Parent, &TabViewHeaderButton), Changed<Interaction>>,
    header_query: Query<(&Parent, &Children), With<TabViewHeader>>,
    view_query: Query<&Children, With<TabView>>,
) -> Result<(), ExplError> {
    let Ok((Interaction::Pressed, parent, TabViewHeaderButton { content })) =
        interaction_query.get_single()
    else {
        return Ok(());
    };
    let (header_parent, header_children) = header_query.get(**parent)?;
    let children = view_query.get(**header_parent)?;
    let mut header_button_iter = header_button_query.iter_many_mut(header_children);
    while let Some((header_button, mut background_color)) = header_button_iter.fetch_next() {
        if header_button.content == *content {
            *background_color = SELECTED.into();
        } else {
            *background_color = NORMAL.into();
        }
    }
    let mut content_iter = content_query.iter_many_mut(&children[1..]);
    while let Some((entity, mut style)) = content_iter.fetch_next() {
        if entity == *content {
            style.display = Display::Flex;
        } else {
            style.display = Display::None;
        }
    }
    Ok(())
}

fn spawn_header_icon(
    parent: &mut ChildBuilder,
    entity: Entity,
    content: &TabViewContent,
    focused: bool,
) {
    parent
        .spawn((
            TabViewHeaderButton { content: entity },
            ButtonBundle {
                background_color: if focused {
                    SELECTED.into()
                } else {
                    NORMAL.into()
                },
                style: Style {
                    margin: UiRect::horizontal(Val::Px(4.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                image: content.icon.clone().into(),
                ..default()
            });
        });
}
