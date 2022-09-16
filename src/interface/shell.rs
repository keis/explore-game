use super::color::{NORMAL, SELECTED};
use super::InterfaceAssets;
use crate::action::GameAction;
use crate::character::Character;
use crate::input::{Action, ActionState};
use crate::map::MapPosition;
use crate::party::Party;
use crate::Turn;
use crate::Zone;
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::{HoverEvent, PickingEvent, Selection};

#[derive(Component)]
pub struct Shell;

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

#[derive(Component)]
pub struct CharacterList;

#[derive(Component)]
pub struct CharacterDisplay {
    character: Entity,
}

fn spawn_toolbar_icon(parent: &mut ChildBuilder, tag: impl Component, image: Handle<Image>) {
    parent
        .spawn_bundle(ButtonBundle {
            color: NORMAL.into(),
            ..default()
        })
        .insert(tag)
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                        ..default()
                    },
                    image: image.into(),
                    ..default()
                })
                .insert(FocusPolicy::Pass);
        });
}

fn spawn_character_display(
    parent: &mut ChildBuilder,
    entity: Entity,
    character: &Character,
    assets: &Res<InterfaceAssets>,
) {
    parent
        .spawn()
        .insert(CharacterDisplay { character: entity })
        .insert_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(40.0)),
                margin: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            color: NORMAL.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                character.name.clone(),
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 28.0,
                    color: Color::WHITE,
                },
            ));
        });
}

pub fn spawn_shell(mut commands: Commands, assets: Res<InterfaceAssets>) {
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
        .insert(Shell)
        .with_children(|parent| {
            parent
                .spawn_bundle(
                    TextBundle::from_section(
                        "Zone: ",
                        TextStyle {
                            font: assets.font.clone(),
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
            parent
                .spawn_bundle(NodeBundle {
                    color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                                flex_direction: FlexDirection::ColumnReverse,
                                margin: UiRect {
                                    right: Val::Px(8.0),
                                    ..default()
                                },
                                ..default()
                            },
                            color: Color::NONE.into(),
                            ..default()
                        })
                        .insert(PartyList);
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                                flex_direction: FlexDirection::ColumnReverse,
                                ..default()
                            },
                            color: Color::NONE.into(),
                            ..default()
                        })
                        .insert(CharacterList);
                });
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
                    spawn_toolbar_icon(parent, CampButton, assets.campfire_icon.clone());
                    spawn_toolbar_icon(parent, BreakCampButton, assets.knapsack_icon.clone());
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
                                    font: assets.font.clone(),
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
                            font: assets.font.clone(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                    parent
                        .spawn_bundle(TextBundle::from_sections([
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
                        ]))
                        .insert(PartyMovementPointsText);
                });
        });
    }
}

pub fn update_character_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    character_list_query: Query<Entity, With<CharacterList>>,
    character_query: Query<(Entity, &Character)>,
    party_query: Query<(&Party, &Selection)>,
    character_display_query: Query<(Entity, &CharacterDisplay)>,
) {
    let character_list = character_list_query.single();

    let characters = party_query
        .iter()
        .filter(|(_, selection)| selection.selected())
        .flat_map(|(party, _)| party.members.iter());
    for (entity, character) in character_query.iter_many(characters) {
        if !character_display_query
            .iter()
            .any(|(_, display)| display.character == entity)
        {
            commands
                .get_or_spawn(character_list)
                .add_children(|parent| {
                    spawn_character_display(parent, entity, character, &assets);
                });
        }
    }

    let characters: Vec<&Entity> = party_query
        .iter()
        .filter(|(_, selection)| selection.selected())
        .flat_map(|(party, _)| party.members.iter())
        .collect();
    for (display_entity, display) in character_display_query.iter() {
        if !characters
            .iter()
            .any(|entity| display.character == **entity)
        {
            commands.entity(display_entity).despawn_recursive();
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

pub fn update_turn_text(mut turn_text_query: Query<&mut Text, With<TurnText>>, turn: Res<Turn>) {
    if turn.is_changed() {
        for mut text in turn_text_query.iter_mut() {
            text.sections[0].value = format!("Turn #{:?}", turn.number);
        }
    }
}

pub fn update_zone_text(
    mut zone_text_query: Query<&mut Text, With<ZoneText>>,
    zone_query: Query<&MapPosition, With<Zone>>,
    mut events: EventReader<PickingEvent>,
) {
    for event in events.iter() {
        if let PickingEvent::Hover(HoverEvent::JustEntered(e)) = event {
            if let Ok(zone_position) = zone_query.get(*e) {
                for mut text in &mut zone_text_query {
                    text.sections[0].value = format!("{:?}", zone_position.0);
                }
            }
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
