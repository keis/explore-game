use super::{
    color::{NORMAL, SELECTED},
    InterfaceAssets,
};
use crate::{
    character::{Character, Movement},
    input::{Action, ActionState},
    party::{Group, Party},
};
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_mod_picking::Selection;

#[derive(Component)]
pub struct PartyList;

#[derive(Component, Debug)]
pub struct PartyDisplay {
    party: Entity,
}

#[derive(Component)]
pub struct PartyMovementPointsText;

#[derive(Bundle)]
pub struct PartyListBundle {
    node_bundle: NodeBundle,
    party_list: PartyList,
}

impl Default for PartyListBundle {
    fn default() -> Self {
        Self {
            node_bundle: NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), Val::Auto),
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
        }
    }
}

fn spawn_party_display(
    parent: &mut ChildBuilder,
    entity: Entity,
    party: &Party,
    party_movement: &Movement,
    assets: &Res<InterfaceAssets>,
) {
    parent
        .spawn((
            PartyDisplay { party: entity },
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(120.0)),
                    margin: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                background_color: NORMAL.into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                party.name.clone(),
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn((
                PartyMovementPointsText,
                TextBundle::from_sections([
                    TextSection::new(
                        "Movement: ",
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::new(
                        format!("{:?}", party_movement.points),
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ),
                ]),
            ));
        });
}

#[allow(clippy::type_complexity)]
pub fn run_if_any_party_changed(
    party_query: Query<Entity, Or<(Changed<Party>, Changed<Group>)>>,
) -> ShouldRun {
    if !party_query.is_empty() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn update_party_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    party_list_query: Query<Entity, With<PartyList>>,
    party_query: Query<(Entity, &Party, &Group, &Movement)>,
    party_display_query: Query<(Entity, &PartyDisplay)>,
) {
    let party_list = party_list_query.single();
    for (entity, party, _, party_movement) in party_query.iter() {
        if party_display_query
            .iter()
            .any(|(_, display)| display.party == entity)
        {
            continue;
        }
        commands.get_or_spawn(party_list).add_children(|parent| {
            spawn_party_display(parent, entity, party, party_movement, &assets);
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

pub fn update_party_selection(
    mut party_display_query: Query<(&PartyDisplay, &mut BackgroundColor)>,
    selection_query: Query<&Selection, (With<Party>, Changed<Selection>)>,
) {
    for (display, mut color) in party_display_query.iter_mut() {
        if let Ok(selection) = selection_query.get(display.party) {
            if selection.selected() {
                *color = SELECTED.into();
            } else {
                *color = NORMAL.into();
            }
        }
    }
}

pub fn update_party_movement_points(
    mut party_movement_points_query: Query<(&mut Text, &Parent), With<PartyMovementPointsText>>,
    party_display_query: Query<&PartyDisplay>,
    party_query: Query<&Movement, (Changed<Movement>, With<Party>)>,
) {
    for (mut text, parent) in party_movement_points_query.iter_mut() {
        if let Ok(party_display) = party_display_query.get(parent.get()) {
            if let Ok(party_movement) = party_query.get(party_display.party) {
                text.sections[1].value = format!("{:?}", party_movement.points);
            }
        }
    }
}

pub fn handle_party_display_interaction(
    action_state_query: Query<&ActionState<Action>>,
    interaction_query: Query<(&Interaction, &PartyDisplay), Changed<Interaction>>,
    mut selection_query: Query<(Entity, &mut Selection), Without<Character>>,
) {
    let action_state = action_state_query.single();
    if let Ok((Interaction::Clicked, display)) = interaction_query.get_single() {
        if let Ok((entity, mut selection)) = selection_query.get_mut(display.party) {
            if action_state.pressed(Action::MultiSelect) {
                let selected = selection.selected();
                selection.set_selected(!selected);
            } else {
                for (e, mut selection) in selection_query.iter_mut() {
                    selection.set_selected(e == entity)
                }
            }
        }
    }
}
