use super::{
    camp::CampListBundle,
    party::PartyListBundle,
    selected::spawn_selected_display,
    style::*,
    styles::{style_button, style_icon, style_root_container},
    tooltip::{spawn_tooltip, TooltipPosition, TooltipTarget},
    InterfaceAssets,
};
use crate::{
    input::{Action, ActionState},
    map::MapPosition,
    terrain::TerrainId,
    turn::Turn,
};
use bevy::prelude::*;
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

fn style_toolbar(style: &mut StyleBuilder) {
    style.align_self(AlignSelf::FlexStart).padding(Val::Px(2.0));
}

fn spawn_toolbar_item(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    tag: impl Component,
    image: Handle<Image>,
    tooltip_text: impl Into<String>,
    keybind_text: Option<impl Into<String>>,
) {
    parent
        .spawn((TooltipTarget, ButtonBundle::default(), tag))
        .with_style(style_button)
        .with_children(|parent| {
            parent
                .spawn(ImageBundle {
                    image: image.into(),
                    ..default()
                })
                .with_style(style_icon);
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
        .spawn((Name::new("Toolbar"), NodeBundle::default()))
        .with_style(style_toolbar)
        .with_children(|parent| {
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::ResumeMove),
                assets.arrow_icon.clone(),
                "Resume move",
                Some("<M>"),
            );
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::Camp),
                assets.campfire_icon.clone(),
                "Make/Enter camp",
                Some("<C>"),
            );
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::BreakCamp),
                assets.cancel_icon.clone(),
                "Break camp",
                None::<&str>,
            );
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::CreateParty),
                assets.knapsack_icon.clone(),
                "Create party",
                None::<&str>,
            );
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::SplitParty),
                assets.back_forth_icon.clone(),
                "Split selected from party",
                None::<&str>,
            );
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::MergeParty),
                assets.contract_icon.clone(),
                "Merge selected parties",
                None::<&str>,
            );
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::CollectCrystals),
                assets.crystals_icon.clone(),
                "Collect crystals",
                None::<&str>,
            );
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::OpenPortal),
                assets.magic_swirl_icon.clone(),
                "Open portal",
                None::<&str>,
            );
            spawn_toolbar_item(
                parent,
                assets,
                ActionButton(Action::EnterPortal),
                assets.portal_icon.clone(),
                "Enter portal",
                None::<&str>,
            );
        });
}

fn style_next_turn_button(style: &mut StyleBuilder) {
    style
        .width(Val::Px(200.0))
        .height(Val::Px(60.0))
        .align_self(AlignSelf::FlexEnd)
        .background_color(Color::srgb(0.4, 0.9, 0.4));
}

fn spawn_next_turn_button(parent: &mut ChildBuilder, assets: &Res<InterfaceAssets>) {
    parent
        .spawn((
            Name::new("Next Turn Button"),
            TooltipTarget,
            ButtonBundle::default(),
            ActionButton(Action::NextTurn),
            TurnButton,
        ))
        .with_style((style_button, style_next_turn_button))
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

fn style_shell_container(style: &mut StyleBuilder) {
    style
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::SpaceBetween)
        .pointer_events(false);
}

fn style_bar(style: &mut StyleBuilder) {
    style
        .width(Val::Percent(100.0))
        .justify_content(JustifyContent::SpaceBetween)
        .pointer_events(false);
}

fn style_outliner(style: &mut StyleBuilder) {
    style
        .flex_direction(FlexDirection::Column)
        .pointer_events(false);
}

fn style_zone_display(style: &mut StyleBuilder) {
    style
        .width(Val::Px(100.0))
        .justify_content(JustifyContent::End);
}

pub fn spawn_shell(mut commands: Commands, assets: Res<InterfaceAssets>) {
    commands
        .spawn((Name::new("Shell Container"), NodeBundle::default(), Shell))
        .with_style((style_root_container, style_shell_container))
        .with_children(|parent| {
            parent
                .spawn((Name::new("Top"), NodeBundle::default()))
                .with_style(style_bar)
                .with_children(|parent| {
                    parent
                        .spawn((NodeBundle::default(), Pickable::IGNORE))
                        .with_children(|parent| {
                            parent
                                .spawn((Name::new("Outliner"), NodeBundle::default()))
                                .with_style(style_outliner)
                                .with_children(|parent| {
                                    parent
                                        .spawn(CampListBundle::default())
                                        .with_style(style_outliner);
                                    parent
                                        .spawn(PartyListBundle::default())
                                        .with_style(style_outliner);
                                });
                        });
                    spawn_toolbar(parent, &assets);
                    parent
                        .spawn((Name::new("Zone Display"), NodeBundle::default()))
                        .with_style(style_zone_display)
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
                .spawn((Name::new("Bottom"), NodeBundle::default()))
                .with_style(style_bar)
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
