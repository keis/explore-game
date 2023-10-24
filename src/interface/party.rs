use super::{
    color::{NORMAL, SELECTED},
    stat::spawn_stat_display,
    InterfaceAssets,
};
use crate::{
    actor::{Character, Group, Movement, Party},
    input::{Action, ActionState, Selection},
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::Pickable;
use expl_databinding::{DataBindingExt, DataBindings};

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
    pickable: Pickable,
}

impl Default for PartyListBundle {
    fn default() -> Self {
        Self {
            node_bundle: NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    margin: UiRect {
                        right: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            party_list: PartyList,
            pickable: Pickable::IGNORE,
        }
    }
}

fn spawn_party_display(
    parent: &mut ChildBuilder,
    entity: Entity,
    party: &Party,
    movement: &Movement,
    group: &Group,
    assets: &Res<InterfaceAssets>,
) {
    parent
        .spawn((
            PartyDisplay { party: entity },
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
                    format!("{}", movement.points),
                );
                spawn_stat_display(
                    parent,
                    assets,
                    entity,
                    PartySizeText,
                    assets.person_icon.clone(),
                    format!("{}", group.members.len()),
                );
                spawn_stat_display(
                    parent,
                    assets,
                    entity,
                    PartyCrystalsText,
                    assets.crystals_icon.clone(),
                    format!("{}", party.crystals),
                );
            });
        });
}

#[allow(clippy::type_complexity)]
pub fn run_if_any_party_changed(
    party_query: Query<Entity, Or<(Changed<Party>, Changed<Group>)>>,
) -> bool {
    !party_query.is_empty()
}

pub fn update_party_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    party_list_query: Query<Entity, With<PartyList>>,
    party_query: Query<(Entity, &Party, &Group, &Movement)>,
    party_display_query: Query<(Entity, &PartyDisplay)>,
) {
    let party_list = party_list_query.single();
    for (entity, party, group, party_movement) in party_query.iter() {
        if party_display_query
            .iter()
            .any(|(_, display)| display.party == entity)
        {
            continue;
        }
        commands.get_or_spawn(party_list).with_children(|parent| {
            spawn_party_display(parent, entity, party, party_movement, group, &assets);
        });
    }

    let party_entities: Vec<Entity> = party_query
        .iter()
        .filter(|(_, _, g, _)| !g.members.is_empty())
        .map(|(e, _, _, _)| e)
        .collect();
    for (display_entity, display) in party_display_query.iter() {
        if !party_entities.iter().any(|&entity| display.party == entity) {
            commands.entity(display_entity).despawn_recursive();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_party_selection(
    mut party_display_query: Query<&mut BackgroundColor, With<PartyDisplay>>,
    selection_query: Query<(&Selection, &DataBindings), (With<Party>, Changed<Selection>)>,
) {
    for (selection, bindings) in &selection_query {
        let mut party_display_iter = party_display_query.iter_many_mut(bindings);
        while let Some(mut background_color) = party_display_iter.fetch_next() {
            *background_color = if selection.is_selected {
                SELECTED
            } else {
                NORMAL
            }
            .into();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_party_movement_points(
    mut party_movement_points_query: Query<&mut Text, With<PartyMovementPointsText>>,
    party_query: Query<(&Movement, &DataBindings), (Changed<Movement>, With<Party>)>,
) {
    for (party_movement, bindings) in &party_query {
        let mut party_movement_points_text_iter =
            party_movement_points_query.iter_many_mut(bindings);
        while let Some(mut text) = party_movement_points_text_iter.fetch_next() {
            text.sections[0].value = format!("{}", party_movement.points);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_party_size(
    mut party_size_text_query: Query<&mut Text, With<PartySizeText>>,
    party_query: Query<(&Group, &DataBindings), (Changed<Group>, With<Party>)>,
) {
    for (group, bindings) in &party_query {
        let mut party_size_text_iter = party_size_text_query.iter_many_mut(bindings);
        while let Some(mut text) = party_size_text_iter.fetch_next() {
            text.sections[0].value = format!("{}", group.members.len());
        }
    }
}

pub fn update_party_crystals(
    mut party_crystals_text_query: Query<&mut Text, With<PartyCrystalsText>>,
    party_query: Query<(&Party, &DataBindings), Changed<Party>>,
) {
    for (party, bindings) in &party_query {
        let mut party_crystals_text_iter = party_crystals_text_query.iter_many_mut(bindings);
        while let Some(mut text) = party_crystals_text_iter.fetch_next() {
            text.sections[0].value = format!("{}", party.crystals);
        }
    }
}

pub fn handle_party_display_interaction(
    action_state: Res<ActionState<Action>>,
    interaction_query: Query<(&Interaction, &PartyDisplay), Changed<Interaction>>,
    mut selection_query: Query<(Entity, &mut Selection), Without<Character>>,
) {
    if let Ok((Interaction::Pressed, display)) = interaction_query.get_single() {
        if let Ok((entity, mut selection)) = selection_query.get_mut(display.party) {
            if action_state.pressed(Action::MultiSelect) {
                let selected = selection.is_selected;
                selection.is_selected = !selected;
            } else {
                for (e, mut selection) in selection_query.iter_mut() {
                    selection.is_selected = e == entity;
                }
            }
        }
    }
}
