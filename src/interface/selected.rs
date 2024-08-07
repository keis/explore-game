use super::{
    camp::spawn_camp_details,
    character::spawn_character_list,
    color::*,
    party::spawn_party_details,
    style::*,
    tabview::{TabView, TabViewContent, TabViewHeader},
    InterfaceAssets,
};
use crate::{
    actor::{Members, Party},
    creature::Movement,
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

fn style_selected_display(style: &mut StyleBuilder) {
    style
        .width(Val::Px(200.0))
        .flex_direction(FlexDirection::Column);
}

fn style_selected_item(style: &mut StyleBuilder) {
    style
        .width(Val::Percent(100.0))
        .height(Val::Px(120.0))
        .margin(Val::Px(2.0))
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(AlignItems::FlexStart)
        .background_color(NORMAL);
}

pub(super) fn spawn_selected_display(parent: &mut ChildBuilder, _assets: &Res<InterfaceAssets>) {
    parent
        .spawn((Name::new("Selected Display"), NodeBundle::default()))
        .with_children(|parent| {
            parent
                .spawn((SelectedDisplay, TabView, NodeBundle::default()))
                .with_style(style_selected_display)
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("Selected Header"),
                        TabViewHeader,
                        NodeBundle::default(),
                    ));
                });
            spawn_character_list(parent);
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
            ButtonBundle::default(),
        ))
        .with_style(style_selected_item)
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
            ButtonBundle::default(),
        ))
        .with_style(style_selected_item)
        .bind_to(entity)
        .with_children(|parent| {
            spawn_camp_details(parent, entity, camp, members, inventory, assets);
        });
}

pub(super) fn handle_deselect_event(
    trigger: Trigger<Deselect>,
    mut commands: Commands,
    inner_query: Query<(Entity, &SelectedInnerDisplay)>,
) {
    if let Some((inner_entity, _)) = inner_query
        .iter()
        .find(|(_, inner)| inner.entity == trigger.entity())
    {
        commands.entity(inner_entity).remove_parent();
        commands.entity(inner_entity).despawn_recursive();
    }
}

pub(super) fn handle_select_event(
    trigger: Trigger<Select>,
    mut commands: Commands,
    display_query: Query<Entity, With<SelectedDisplay>>,
    inner_query: Query<(Entity, &SelectedInnerDisplay)>,
    party_query: Query<(&Party, &Members, &Movement, &Inventory)>,
    camp_query: Query<(&Camp, &Members, &Inventory)>,
    assets: Res<InterfaceAssets>,
) {
    let display = display_query.single();
    let entity = trigger.entity();
    if !inner_query.iter().any(|(_, inner)| inner.entity == entity) {
        if let Ok((party, members, movement, inventory)) = party_query.get(entity) {
            commands.get_or_spawn(display).with_children(|parent| {
                spawn_party_display(parent, &assets, entity, party, members, movement, inventory);
            });
            return;
        }

        if let Ok((camp, members, inventory)) = camp_query.get(entity) {
            commands.get_or_spawn(display).with_children(|parent| {
                spawn_camp_display(parent, &assets, entity, camp, members, inventory);
            });
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
