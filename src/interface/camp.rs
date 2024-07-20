use super::{
    color::{NORMAL, SELECTED},
    party::PartySizeText,
    stat::spawn_stat_display,
    style::*,
    styles::{style_button, style_icon},
    InterfaceAssets,
};
use crate::{
    actor::{Character, Members},
    input::{Selection, SelectionUpdate},
    inventory::Inventory,
    structure::Camp,
};
use bevy::prelude::*;
use expl_databinding::{DataBindingExt, DataBindingUpdate};

#[derive(Component)]
pub struct CampList;

#[derive(Component, Debug)]
pub struct CampDisplay {
    camp: Entity,
}

#[derive(Component)]
pub struct CampCrystalsText;

#[derive(Bundle)]
pub struct CampListBundle {
    node_bundle: NodeBundle,
    camp_list: CampList,
}

impl Default for CampListBundle {
    fn default() -> Self {
        Self {
            node_bundle: NodeBundle::default(),
            camp_list: CampList,
        }
    }
}

fn spawn_camp_display(parent: &mut ChildBuilder, entity: Entity, assets: &Res<InterfaceAssets>) {
    parent
        .spawn((CampDisplay { camp: entity }, ButtonBundle::default()))
        .with_style(style_button)
        .bind_to(entity)
        .with_children(|parent| {
            parent
                .spawn(ImageBundle {
                    image: assets.campfire_icon.clone().into(),
                    ..default()
                })
                .with_style(style_icon);
        });
}

pub fn spawn_camp_details(
    parent: &mut ChildBuilder,
    entity: Entity,
    camp: &Camp,
    members: &Members,
    inventory: &Inventory,
    assets: &Res<InterfaceAssets>,
) {
    parent.spawn(TextBundle::from_section(
        camp.name.clone(),
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
            CampCrystalsText,
            assets.crystals_icon.clone(),
            format!("{}", inventory.count_item(Inventory::CRYSTAL)),
        );
        spawn_stat_display(
            parent,
            assets,
            entity,
            PartySizeText,
            assets.person_icon.clone(),
            format!("{}", members.len()),
        );
    });
}

pub fn update_camp_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    camp_list_query: Query<Entity, With<CampList>>,
    camp_query: Query<Entity, Added<Camp>>,
    camp_display_query: Query<(Entity, &CampDisplay)>,
) {
    let camp_list = camp_list_query.single();
    for entity in camp_query.iter() {
        if camp_display_query
            .iter()
            .any(|(_, display)| display.camp == entity)
        {
            continue;
        }
        commands.get_or_spawn(camp_list).with_children(|parent| {
            spawn_camp_display(parent, entity, &assets);
        });
    }
}

pub(super) fn remove_despawned(
    mut commands: Commands,
    mut removed_camp: RemovedComponents<Camp>,
    camp_display_query: Query<(Entity, &CampDisplay)>,
) {
    for entity in removed_camp.read() {
        if let Some((display_entity, _)) = camp_display_query
            .iter()
            .find(|(_, display)| display.camp == entity)
        {
            commands.entity(display_entity).remove_parent();
            commands.entity(display_entity).despawn_recursive();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_camp_selection(
    mut data_binding_update: DataBindingUpdate<
        &Selection,
        &mut BackgroundColor,
        (Changed<Selection>, With<Camp>),
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

pub fn update_camp_crystals(
    mut data_binding_update: DataBindingUpdate<
        &Inventory,
        &mut Text,
        (Changed<Inventory>, With<Camp>),
    >,
) {
    data_binding_update.for_each(|inventory, text| {
        text.sections[0].value = format!("{}", inventory.count_item(Inventory::CRYSTAL));
    });
}

pub fn handle_camp_display_interaction(
    interaction_query: Query<(&Interaction, &CampDisplay), Changed<Interaction>>,
    mut selection: SelectionUpdate<Without<Character>>,
) {
    if let Ok((Interaction::Pressed, display)) = interaction_query.get_single() {
        selection.toggle(display.camp);
    }
}
