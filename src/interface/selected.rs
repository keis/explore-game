use super::{
    camp::spawn_camp_details,
    character::CharacterListBundle,
    color::*,
    party::spawn_party_details,
    tabview::{TabView, TabViewContent, TabViewHeader},
    InterfaceAssets,
};
use crate::{
    actor::{Members, Movement, Party},
    input::{Deselect, Select, Selection},
    inventory::Inventory,
    structure::Camp,
};
use bevy::prelude::*;
use expl_databinding::DataBindingExt;

#[derive(Component)]
pub(super) struct SelectedDisplay;

// Back-pointer to data binding, common enough to generalise?
#[derive(Component)]
pub(super) struct SelectedInnerDisplay {
    entity: Entity,
}

pub(super) fn spawn_selected_display(parent: &mut ChildBuilder, _assets: &Res<InterfaceAssets>) {
    parent
        .spawn((Name::new("Selected Display"), NodeBundle::default()))
        .with_children(|parent| {
            parent
                .spawn((
                    SelectedDisplay,
                    TabView,
                    NodeBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("Selected Header"),
                        TabViewHeader,
                        NodeBundle::default(),
                    ));
                });
            parent.spawn(CharacterListBundle::default());
        });
}

fn spawn_party_display(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    entity: Entity,
    party: &Party,
    members: &Members,
    movement: &Movement,
    inventory: &Inventory,
) -> Entity {
    parent
        .spawn((
            SelectedInnerDisplay { entity },
            TabViewContent {
                icon: assets.brutal_helm_icon.clone(),
            },
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(120.0),
                    margin: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                background_color: NORMAL.into(),
                ..default()
            },
        ))
        .bind_to(entity)
        .with_children(|parent| {
            spawn_party_details(parent, entity, party, movement, members, inventory, assets);
        })
        .id()
}

fn spawn_camp_display(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    entity: Entity,
    camp: &Camp,
    members: &Members,
    inventory: &Inventory,
) {
    parent
        .spawn((
            SelectedInnerDisplay { entity },
            TabViewContent {
                icon: assets.campfire_icon.clone(),
            },
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(120.0),
                    margin: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                background_color: NORMAL.into(),
                ..default()
            },
        ))
        .bind_to(entity)
        .with_children(|parent| {
            spawn_camp_details(parent, entity, camp, members, inventory, assets);
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update_selected(
    mut commands: Commands,
    mut deselect_events: EventReader<Deselect>,
    mut select_events: EventReader<Select>,
    display_query: Query<Entity, With<SelectedDisplay>>,
    inner_query: Query<(Entity, &SelectedInnerDisplay)>,
    party_query: Query<(&Party, &Members, &Movement, &Inventory)>,
    camp_query: Query<(&Camp, &Members, &Inventory)>,
    assets: Res<InterfaceAssets>,
) {
    let display = display_query.single();
    for Deselect(entity) in deselect_events.read() {
        if let Some((inner_entity, _)) = inner_query
            .iter()
            .find(|(_, inner)| inner.entity == *entity)
        {
            commands.entity(inner_entity).remove_parent();
            commands.entity(inner_entity).despawn_recursive();
        }
    }
    for Select(entity) in select_events.read() {
        if !inner_query.iter().any(|(_, inner)| inner.entity == *entity) {
            if let Ok((party, members, movement, inventory)) = party_query.get(*entity) {
                commands.get_or_spawn(display).with_children(|parent| {
                    spawn_party_display(
                        parent, &assets, *entity, party, members, movement, inventory,
                    );
                });
                continue;
            }

            if let Ok((camp, members, inventory)) = camp_query.get(*entity) {
                commands.get_or_spawn(display).with_children(|parent| {
                    spawn_camp_display(parent, &assets, *entity, camp, members, inventory);
                });
                continue;
            }
        }
    }
}

pub(super) fn remove_despawned(
    mut commands: Commands,
    mut removed_selection: RemovedComponents<Selection>,
    inner_query: Query<(Entity, &SelectedInnerDisplay)>,
) {
    for entity in removed_selection.read() {
        if let Some((inner_entity, _)) =
            inner_query.iter().find(|(_, inner)| inner.entity == entity)
        {
            commands.entity(inner_entity).remove_parent();
            commands.entity(inner_entity).despawn_recursive();
        }
    }
}
