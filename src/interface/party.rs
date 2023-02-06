use super::{
    color::{NORMAL, SELECTED},
    InterfaceAssets,
};
use crate::{
    input::{Action, ActionState},
    party::Party,
};
use bevy::prelude::*;
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
                    size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
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

pub fn update_party_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    party_list_query: Query<Entity, With<PartyList>>,
    party_query: Query<(Entity, &Party)>,
    party_display_query: Query<&PartyDisplay>,
) {
    let party_list = party_list_query.single();
    for (entity, party) in party_query.iter() {
        if party_display_query
            .iter()
            .any(|display| display.party == entity)
        {
            continue;
        }
        commands.get_or_spawn(party_list).add_children(|parent| {
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
                                format!("{:?}", party.movement_points),
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            ),
                        ]),
                    ));
                });
        });
    }
}

pub fn update_party_selection(
    mut party_display_query: Query<(&PartyDisplay, &mut BackgroundColor)>,
    party_query: Query<&Selection, (With<Party>, Changed<Selection>)>,
) {
    for (party_display, mut color) in party_display_query.iter_mut() {
        if let Ok(selection) = party_query.get(party_display.party) {
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
    party_query: Query<&Party, Changed<Party>>,
) {
    for (mut text, parent) in party_movement_points_query.iter_mut() {
        if let Ok(party_display) = party_display_query.get(parent.get()) {
            if let Ok(party) = party_query.get(party_display.party) {
                text.sections[1].value = format!("{:?}", party.movement_points);
            }
        }
    }
}

pub fn handle_party_display_interaction(
    action_state_query: Query<&ActionState<Action>>,
    interaction_query: Query<(&Interaction, &PartyDisplay), Changed<Interaction>>,
    mut party_query: Query<(Entity, &Party, &mut Selection)>,
) {
    let action_state = action_state_query.single();
    if let Ok((Interaction::Clicked, partydisplay)) = interaction_query.get_single() {
        if let Ok((entity, party, mut selection)) = party_query.get_mut(partydisplay.party) {
            info!("Clicked party {:?}", party);
            if action_state.pressed(Action::MultiSelect) {
                let selected = selection.selected();
                selection.set_selected(!selected);
            } else {
                for (e, _, mut selection) in party_query.iter_mut() {
                    selection.set_selected(e == entity)
                }
            }
        }
    }
}
