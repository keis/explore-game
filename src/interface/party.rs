use super::{
    color::{NORMAL, SELECTED},
    stat::spawn_stat_display,
    style::*,
    styles::{style_button, style_icon},
    InterfaceAssets,
};
use crate::{
    actor::{Character, Members, Party},
    creature::Movement,
    input::{Selection, SelectionUpdate},
    inventory::Inventory,
};
use bevy::prelude::*;
use expl_databinding::{DataBindingExt, DataBindingUpdate};

#[derive(Component)]
pub struct PartyList;

#[derive(Component, Debug)]
pub struct PartyDisplay {
    party: Entity,
}

#[derive(Component)]
pub struct PartyMovementPointsText;

#[derive(Component)]
pub struct PartySizeText;

#[derive(Component)]
pub struct PartyCrystalsText;

#[derive(Bundle)]
pub struct PartyListBundle {
    node_bundle: NodeBundle,
    party_list: PartyList,
}

impl Default for PartyListBundle {
    fn default() -> Self {
        Self {
            node_bundle: NodeBundle::default(),
            party_list: PartyList,
        }
    }
}

fn spawn_party_display(parent: &mut ChildBuilder, entity: Entity, assets: &Res<InterfaceAssets>) {
    parent
        .spawn((PartyDisplay { party: entity }, ButtonBundle::default()))
        .with_style(style_button)
        .bind_to(entity)
        .with_children(|parent| {
            parent
                .spawn(ImageBundle {
                    image: assets.brutal_helm_icon.clone().into(),
                    ..default()
                })
                .with_style(style_icon);
        });
}

pub fn spawn_party_details(
    parent: &mut ChildBuilder,
    entity: Entity,
    party: &Party,
    movement: &Movement,
    members: &Members,
    inventory: &Inventory,
    assets: &Res<InterfaceAssets>,
) {
    parent.spawn(TextBundle::from_section(
        party.name.clone(),
        TextStyle {
            font: assets.font.clone(),
            font_size: 32.0,
            color: Color::WHITE,
        },
    ));
    parent.spawn(NodeBundle::default()).with_children(|parent| {
        spawn_stat_display(
            parent,
            assets,
            entity,
            PartyMovementPointsText,
            assets.footsteps_icon.clone(),
            format!("{}", movement.current),
        );
        spawn_stat_display(
            parent,
            assets,
            entity,
            PartySizeText,
            assets.person_icon.clone(),
            format!("{}", members.len()),
        );
        spawn_stat_display(
            parent,
            assets,
            entity,
            PartyCrystalsText,
            assets.crystals_icon.clone(),
            format!("{}", inventory.count_item(Inventory::CRYSTAL)),
        );
    });
}

pub fn update_party_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    party_list_query: Query<Entity, With<PartyList>>,
    party_query: Query<Entity, Added<Party>>,
    party_display_query: Query<(Entity, &PartyDisplay)>,
) {
    let party_list = party_list_query.single();
    for entity in party_query.iter() {
        if party_display_query
            .iter()
            .any(|(_, display)| display.party == entity)
        {
            continue;
        }
        commands.get_or_spawn(party_list).with_children(|parent| {
            spawn_party_display(parent, entity, &assets);
        });
    }
}

pub(super) fn remove_despawned(
    mut commands: Commands,
    mut removed_party: RemovedComponents<Party>,
    party_display_query: Query<(Entity, &PartyDisplay)>,
) {
    for entity in removed_party.read() {
        if let Some((display_entity, _)) = party_display_query
            .iter()
            .find(|(_, display)| display.party == entity)
        {
            commands.entity(display_entity).remove_parent();
            commands.entity(display_entity).despawn_recursive();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_party_selection(
    mut data_binding_update: DataBindingUpdate<
        &Selection,
        &mut BackgroundColor,
        (Changed<Selection>, With<Party>),
    >,
) {
    data_binding_update.for_each(|selection, background_color| {
        **background_color = if selection.is_selected {
            SELECTED
        } else {
            NORMAL
        }
        .into();
    });
}

#[allow(clippy::type_complexity)]
pub fn update_party_movement_points(
    mut data_binding_update: DataBindingUpdate<
        &Movement,
        &mut Text,
        (Changed<Movement>, With<Party>),
    >,
) {
    data_binding_update.for_each(|movement, text| {
        text.sections[0].value = format!("{}", movement.current);
    });
}

pub fn update_party_size(
    mut data_binding_update: DataBindingUpdate<
        &Members,
        &mut Text,
        (Changed<Members>, With<Party>),
    >,
) {
    data_binding_update.for_each(|members, text| {
        text.sections[0].value = format!("{}", members.len());
    });
}

pub fn update_party_crystals(
    mut data_binding_update: DataBindingUpdate<
        &Inventory,
        &mut Text,
        (Changed<Inventory>, With<Party>),
    >,
) {
    data_binding_update.for_each(|inventory, text| {
        text.sections[0].value = format!("{}", inventory.count_item(Inventory::CRYSTAL));
    });
}

pub fn handle_party_display_interaction(
    interaction_query: Query<(&Interaction, &PartyDisplay), Changed<Interaction>>,
    mut selection: SelectionUpdate<Without<Character>>,
) {
    if let Ok((Interaction::Pressed, display)) = interaction_query.get_single() {
        selection.toggle(display.party);
    }
}
