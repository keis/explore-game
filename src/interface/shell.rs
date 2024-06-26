use super::{
    camp::CampListBundle,
    color::NORMAL,
    party::PartyListBundle,
    selected::spawn_selected_display,
    tooltip::{spawn_tooltip, TooltipPosition, TooltipTarget},
    InterfaceAssets,
};
use crate::{
    input::{Action, ActionState},
    map::MapPosition,
    terrain::TerrainId,
    turn::Turn,
};
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::prelude::{Pickable, PickingInteraction};

#[derive(Component)]
pub struct Shell;

#[derive(Component)]
pub struct ZoneText;

#[derive(Component)]
pub struct TurnButton;

#[derive(Component)]
pub struct TurnText;

#[derive(Component)]
pub struct ActionButton(Action);

fn spawn_toolbar_icon(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    tag: impl Component,
    image: Handle<Image>,
    tooltip_text: impl Into<String>,
    keybind_text: Option<impl Into<String>>,
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
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                image: image.into(),
                focus_policy: FocusPolicy::Pass,
                ..default()
            });
            spawn_tooltip(
                parent,
                assets,
                TooltipPosition::Below,
                tooltip_text,
                keybind_text,
            )
        });
}

fn spawn_toolbar(parent: &mut ChildBuilder, assets: &Res<InterfaceAssets>) {
    parent
        .spawn((
            Name::new("Toolbar"),
            NodeBundle {
                style: Style {
                    align_self: AlignSelf::FlexStart,
                    padding: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::ResumeMove),
                assets.arrow_icon.clone(),
                "Resume move",
                Some("<M>"),
            );
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::Camp),
                assets.campfire_icon.clone(),
                "Make/Enter camp",
                Some("<C>"),
            );
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::BreakCamp),
                assets.cancel_icon.clone(),
                "Break camp",
                None::<&str>,
            );
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::CreateParty),
                assets.knapsack_icon.clone(),
                "Create party",
                None::<&str>,
            );
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::SplitParty),
                assets.back_forth_icon.clone(),
                "Split selected from party",
                None::<&str>,
            );
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::MergeParty),
                assets.contract_icon.clone(),
                "Merge selected parties",
                None::<&str>,
            );
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::CollectCrystals),
                assets.crystals_icon.clone(),
                "Collect crystals",
                None::<&str>,
            );
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::OpenPortal),
                assets.magic_swirl_icon.clone(),
                "Open portal",
                None::<&str>,
            );
            spawn_toolbar_icon(
                parent,
                assets,
                ActionButton(Action::EnterPortal),
                assets.portal_icon.clone(),
                "Enter portal",
                None::<&str>,
            );
        });
}

fn spawn_next_turn_button(parent: &mut ChildBuilder, assets: &Res<InterfaceAssets>) {
    parent
        .spawn((
            Name::new("Next Turn Button"),
            TooltipTarget,
            ButtonBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::FlexEnd,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgb(0.4, 0.9, 0.4).into(),
                ..default()
            },
            ActionButton(Action::NextTurn),
            TurnButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Next Turn Button Text"),
                TurnText,
                TextBundle::from_section(
                    "Turn ?",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 32.0,
                        color: Color::WHITE,
                    },
                )
                .with_text_justify(JustifyText::Center)
                .with_style(Style { ..default() }),
            ));
            spawn_tooltip(
                parent,
                assets,
                TooltipPosition::Above,
                "Next turn",
                Some("<Return>"),
            );
        });
}

pub fn spawn_shell(mut commands: Commands, assets: Res<InterfaceAssets>) {
    commands
        .spawn((
            Name::new("Shell Container"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                visibility: Visibility::Hidden,
                ..default()
            },
            Pickable::IGNORE,
            Shell,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Top"),
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        focus_policy: FocusPolicy::Pass,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((NodeBundle::default(), Pickable::IGNORE))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    NodeBundle {
                                        style: Style {
                                            flex_direction: FlexDirection::Column,
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ))
                                .with_children(|parent| {
                                    parent.spawn(CampListBundle::default());
                                    parent.spawn(PartyListBundle::default());
                                });
                        });
                    spawn_toolbar(parent, &assets);
                    parent
                        .spawn((
                            Name::new("Zone Display"),
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(100.0),
                                    justify_content: JustifyContent::End,
                                    ..default()
                                },
                                ..default()
                            },
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
                                .with_text_justify(JustifyText::Center),
                            ));
                        });
                });
            parent
                .spawn((
                    Name::new("Bottom"),
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        focus_policy: FocusPolicy::Pass,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|parent| {
                    spawn_selected_display(parent, &assets);
                    spawn_next_turn_button(parent, &assets);
                });
        });
}

pub fn show_shell(mut shell_query: Query<&mut Visibility, With<Shell>>) {
    let mut shell_visibility = shell_query.single_mut();
    *shell_visibility = Visibility::Inherited;
}

pub fn hide_shell(mut shell_query: Query<&mut Visibility, With<Shell>>) {
    let mut shell_visibility = shell_query.single_mut();
    *shell_visibility = Visibility::Hidden;
}

pub fn update_turn_text(mut turn_text_query: Query<&mut Text, With<TurnText>>, turn: Res<Turn>) {
    if turn.is_changed() {
        for mut text in turn_text_query.iter_mut() {
            text.sections[0].value = format!("Turn #{:?}", **turn);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_zone_text(
    mut zone_text_query: Query<&mut Text, With<ZoneText>>,
    zone_query: Query<
        (&MapPosition, &PickingInteraction),
        (With<TerrainId>, Changed<PickingInteraction>),
    >,
) {
    for (zone_position, _) in zone_query
        .iter()
        .filter(|(_, interaction)| **interaction == PickingInteraction::Hovered)
    {
        for mut text in &mut zone_text_query {
            text.sections[0].value = format!("{}", zone_position.0);
        }
    }
}

pub fn handle_action_button_interaction(
    interaction_query: Query<(&ActionButton, &Interaction), Changed<Interaction>>,
    mut action_state: ResMut<ActionState<Action>>,
) {
    for ActionButton(action) in interaction_query
        .iter()
        .filter(|(_, interaction)| **interaction == Interaction::Pressed)
        .map(|(action, _)| action)
    {
        action_state.press(action);
    }
}
