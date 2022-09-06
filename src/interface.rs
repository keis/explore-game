use crate::action::GameAction;
use crate::input::{Action, ActionState};
use crate::party::Party;
use crate::Turn;
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::Selection;

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_interface)
            .add_system(update_party_list)
            .add_system(update_party_selection)
            .add_system(update_party_movement_points)
            .add_system(handle_party_display_interaction)
            .add_system(update_turn_text)
            .add_system(handle_turn_button_interaction)
            .add_system(handle_camp_button_interaction)
            .add_system(handle_break_camp_button_interaction);
    }
}

#[derive(Component)]
pub struct ZoneText;

#[derive(Component)]
pub struct TurnButton;

#[derive(Component)]
pub struct TurnText;

#[derive(Component)]
pub struct PartyList;

#[derive(Component)]
pub struct PartyMovementPointsText;

#[derive(Component)]
pub struct CampButton;

#[derive(Component)]
pub struct BreakCampButton;

#[derive(Component, Debug)]
pub struct PartyDisplay {
    party: Entity,
}

const NORMAL: Color = Color::rgb(0.20, 0.20, 0.20);
const SELECTED: Color = Color::rgb(0.75, 0.50, 0.50);

fn spawn_interface(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(
            TextBundle::from_section(
                "Zone: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            )
            .with_text_alignment(TextAlignment::TOP_CENTER)
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ZoneText);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexEnd,
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .insert(PartyList);
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        align_self: AlignSelf::FlexEnd,
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(ButtonBundle {
                            color: NORMAL.into(),
                            ..default()
                        })
                        .insert(CampButton)
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(ImageBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                                        ..default()
                                    },
                                    image: asset_server.load("icons/campfire.png").into(),
                                    ..default()
                                })
                                .insert(FocusPolicy::Pass);
                        });
                    parent
                        .spawn_bundle(ButtonBundle {
                            color: NORMAL.into(),
                            ..default()
                        })
                        .insert(BreakCampButton)
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(ImageBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                                        ..default()
                                    },
                                    image: asset_server.load("icons/knapsack.png").into(),
                                    ..default()
                                })
                                .insert(FocusPolicy::Pass);
                        });
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(60.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    color: Color::rgb(0.4, 0.9, 0.4).into(),
                    ..default()
                })
                .insert(TurnButton)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(
                            TextBundle::from_section(
                                "Turn ?",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            )
                            .with_text_alignment(TextAlignment::CENTER)
                            .with_style(Style { ..default() }),
                        )
                        .insert(TurnText);
                });
        });
}

fn update_party_list(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    party_list_query: Query<Entity, With<PartyList>>,
    party_query: Query<(Entity, &Party)>,
    party_display_query: Query<&PartyDisplay>,
) {
    let party_list = party_list_query.single();
    for (entity, party) in party_query.iter() {
        if !party_display_query
            .iter()
            .any(|display| display.party == entity)
        {
            commands.get_or_spawn(party_list).add_children(|parent| {
                parent
                    .spawn()
                    .insert(PartyDisplay { party: entity })
                    .insert_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Px(120.0)),
                            margin: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::ColumnReverse,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        color: NORMAL.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            party.name.clone(),
                            TextStyle {
                                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                font_size: 32.0,
                                color: Color::WHITE,
                            },
                        ));
                        parent
                            .spawn_bundle(TextBundle::from_sections([
                                TextSection::new(
                                    "Movement: ",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                        font_size: 32.0,
                                        color: Color::WHITE,
                                    },
                                ),
                                TextSection::new(
                                    format!("{:?}", party.movement_points),
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                        font_size: 32.0,
                                        color: Color::WHITE,
                                    },
                                ),
                            ]))
                            .insert(PartyMovementPointsText);
                    });
            });
        }
    }
}

pub fn update_party_selection(
    mut party_display_query: Query<(&PartyDisplay, &mut UiColor)>,
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

pub fn update_turn_text(mut turn_text_query: Query<&mut Text, With<TurnText>>, turn: Res<Turn>) {
    if turn.is_changed() {
        for mut text in turn_text_query.iter_mut() {
            text.sections[0].value = format!("Turn #{:?}", turn.number);
        }
    }
}

pub fn handle_turn_button_interaction(
    interaction_query: Query<&Interaction, (With<TurnButton>, Changed<Interaction>)>,
    mut turn: ResMut<Turn>,
) {
    if let Ok(Interaction::Clicked) = interaction_query.get_single() {
        turn.number += 1;
    }
}

pub fn handle_camp_button_interaction(
    interaction_query: Query<&Interaction, (With<CampButton>, Changed<Interaction>)>,
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    if let Ok(Interaction::Clicked) = interaction_query.get_single() {
        for (entity, _) in party_query.iter().filter(|(_, s)| s.selected()) {
            game_action_event.send(GameAction::MakeCamp(entity));
        }
    }
}

pub fn handle_break_camp_button_interaction(
    interaction_query: Query<&Interaction, (With<BreakCampButton>, Changed<Interaction>)>,
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    if let Ok(Interaction::Clicked) = interaction_query.get_single() {
        for (entity, _) in party_query.iter().filter(|(_, s)| s.selected()) {
            game_action_event.send(GameAction::BreakCamp(entity));
        }
    }
}
