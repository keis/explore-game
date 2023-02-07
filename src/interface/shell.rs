use super::{
    character::CharacterListBundle, color::NORMAL, party::PartyListBundle, InterfaceAssets,
};
use crate::{
    action::GameAction,
    map::{MapPosition, Zone},
    party::Party,
    turn::Turn,
};
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
pub struct MoveButton;

#[derive(Component)]
pub struct CampButton;

#[derive(Component)]
pub struct BreakCampButton;

fn spawn_toolbar_icon(parent: &mut ChildBuilder, tag: impl Component, image: Handle<Image>) {
    parent
        .spawn((
            ButtonBundle {
                background_color: NORMAL.into(),
                ..default()
            },
            tag,
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                    ..default()
                },
                image: image.into(),
                focus_policy: FocusPolicy::Pass,
                ..default()
            });
        });
}

pub fn spawn_shell(mut commands: Commands, assets: Res<InterfaceAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            Shell,
        ))
        .with_children(|parent| {
            parent.spawn((
                ZoneText,
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
            ));
            parent
                .spawn(NodeBundle {
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(PartyListBundle::default());
                    parent.spawn(CharacterListBundle::default());
                });
            parent
                .spawn(NodeBundle {
                    style: Style {
                        align_self: AlignSelf::FlexStart,
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    spawn_toolbar_icon(parent, MoveButton, assets.arrow_icon.clone());
                    spawn_toolbar_icon(parent, CampButton, assets.campfire_icon.clone());
                    spawn_toolbar_icon(parent, BreakCampButton, assets.knapsack_icon.clone());
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(200.0), Val::Px(60.0)),
                            align_self: AlignSelf::FlexEnd,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: Color::rgb(0.4, 0.9, 0.4).into(),
                        ..default()
                    },
                    TurnButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TurnText,
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
                    ));
                });
        });
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

pub fn handle_move_button_interaction(
    interaction_query: Query<&Interaction, (With<MoveButton>, Changed<Interaction>)>,
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    if let Ok(Interaction::Clicked) = interaction_query.get_single() {
        for (entity, _) in party_query.iter().filter(|(_, s)| s.selected()) {
            game_action_event.send(GameAction::ResumeMove(entity));
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
