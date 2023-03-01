use super::{
    camp::CampListBundle,
    character::CharacterListBundle,
    color::NORMAL,
    party::PartyListBundle,
    tooltip::{spawn_tooltip, TooltipTarget},
    InterfaceAssets,
};
use crate::{
    action::GameAction,
    camp::Camp,
    character::Character,
    map::{GameMap, MapPosition, MapPresence, Zone},
    party::{Group, Party},
    turn::Turn,
};
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::{HoverEvent, PickingEvent, Selection};
use smallvec::SmallVec;

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

#[derive(Component)]
pub struct CreatePartyButton;

#[derive(Component)]
pub struct SplitPartyButton;

#[derive(Component)]
pub struct MergePartyButton;

#[derive(Component)]
pub struct CollectCrystalsButton;

fn spawn_toolbar_icon(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    tag: impl Component,
    image: Handle<Image>,
    tooltip: impl Into<String>,
) {
    parent
        .spawn((
            TooltipTarget,
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
            spawn_tooltip(parent, assets, tooltip)
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
                .spawn(NodeBundle { ..default() })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(CampListBundle::default());
                            parent.spawn(PartyListBundle::default());
                        });
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
                    spawn_toolbar_icon(
                        parent,
                        &assets,
                        MoveButton,
                        assets.arrow_icon.clone(),
                        "Continue movement",
                    );
                    spawn_toolbar_icon(
                        parent,
                        &assets,
                        CampButton,
                        assets.campfire_icon.clone(),
                        "Make/Enter camp",
                    );
                    spawn_toolbar_icon(
                        parent,
                        &assets,
                        BreakCampButton,
                        assets.cancel_icon.clone(),
                        "Break camp",
                    );
                    spawn_toolbar_icon(
                        parent,
                        &assets,
                        CreatePartyButton,
                        assets.knapsack_icon.clone(),
                        "Create party",
                    );
                    spawn_toolbar_icon(
                        parent,
                        &assets,
                        SplitPartyButton,
                        assets.back_forth_icon.clone(),
                        "Split selected from party",
                    );
                    spawn_toolbar_icon(
                        parent,
                        &assets,
                        MergePartyButton,
                        assets.contract_icon.clone(),
                        "Merge selected parties",
                    );
                    spawn_toolbar_icon(
                        parent,
                        &assets,
                        CollectCrystalsButton,
                        assets.crystals_icon.clone(),
                        "Collect crystals",
                    );
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
    party_query: Query<(Entity, &MapPresence, &Selection), With<Party>>,
    map_query: Query<&GameMap>,
    camp_query: Query<Entity, With<Camp>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    if let Ok(Interaction::Clicked) = interaction_query.get_single() {
        for (entity, presence, _) in party_query.iter().filter(|(_, _, s)| s.selected()) {
            let Ok(map) = map_query.get(presence.map) else { continue };
            if let Some(camp_entity) = camp_query.iter_many(map.presence(presence.position)).next()
            {
                game_action_event.send(GameAction::EnterCamp(entity, camp_entity));
            } else {
                game_action_event.send(GameAction::MakeCamp(entity));
            }
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

pub fn handle_create_party_button_interaction(
    interaction_query: Query<&Interaction, (With<CreatePartyButton>, Changed<Interaction>)>,
    camp_query: Query<(Entity, &Group, &Selection), With<Camp>>,
    character_query: Query<(Entity, &Selection), With<Character>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    let Ok(Interaction::Clicked) = interaction_query.get_single() else { return };
    for (entity, group, _) in camp_query.iter().filter(|(_, _, s)| s.selected()) {
        let selected: SmallVec<[Entity; 8]> = character_query
            .iter_many(&group.members)
            .filter(|(_, s)| s.selected())
            .map(|(e, _)| e)
            .collect();
        if !selected.is_empty() {
            game_action_event.send(GameAction::CreatePartyFromCamp(entity, selected));
        }
    }
}

pub fn handle_split_party_button_interaction(
    interaction_query: Query<&Interaction, (With<SplitPartyButton>, Changed<Interaction>)>,
    party_query: Query<(Entity, &Group, &Selection), With<Party>>,
    character_query: Query<(Entity, &Selection), With<Character>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    let Ok(Interaction::Clicked) = interaction_query.get_single() else { return };
    for (entity, group, _) in party_query.iter().filter(|(_, _, s)| s.selected()) {
        let selected: SmallVec<[Entity; 8]> = character_query
            .iter_many(&group.members)
            .filter(|(_, s)| s.selected())
            .map(|(e, _)| e)
            .collect();
        if !selected.is_empty() {
            game_action_event.send(GameAction::SplitParty(entity, selected));
        }
    }
}

pub fn handle_merge_party_button_interaction(
    interaction_query: Query<&Interaction, (With<MergePartyButton>, Changed<Interaction>)>,
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    let Ok(Interaction::Clicked) = interaction_query.get_single() else { return };
    let selected_parties: SmallVec<[Entity; 8]> = party_query
        .iter()
        .filter(|(_, s)| s.selected())
        .map(|(e, _)| e)
        .collect();
    if !selected_parties.is_empty() {
        game_action_event.send(GameAction::MergeParty(selected_parties));
    }
}

pub fn handle_collect_crystals_button_interaction(
    interaction_query: Query<&Interaction, (With<CollectCrystalsButton>, Changed<Interaction>)>,
    party_query: Query<(Entity, &Selection), With<Party>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    let Ok(Interaction::Clicked) = interaction_query.get_single() else { return };
    for (party, _) in party_query.iter().filter(|(_, s)| s.selected()) {
        game_action_event.send(GameAction::CollectCrystals(party));
    }
}
