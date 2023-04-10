use super::color::{BACKGROUND, MENU, NORMAL};
use super::{shell::Shell, InterfaceAssets};
use crate::action::GameAction;
use crate::input::{Action, ActionState};
use bevy::prelude::*;

#[derive(Component)]
pub struct MenuLayer;

#[derive(Component)]
pub struct MenuItemSave;

#[derive(Component)]
pub struct MenuItemQuit;

fn spawn_menu_item(
    parent: &mut ChildBuilder,
    assets: &Res<InterfaceAssets>,
    tag: impl Component,
    text: &str,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(50.0)),
                    margin: UiRect {
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    ..default()
                },
                background_color: NORMAL.into(),
                ..default()
            },
            tag,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ));
        });
}

pub fn spawn_menu(mut commands: Commands, assets: Res<InterfaceAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    ..default()
                },
                background_color: BACKGROUND.into(),
                visibility: Visibility::Hidden,
                ..default()
            },
            MenuLayer,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(300.0), Val::Px(400.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: MENU.into(),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_menu_item(parent, &assets, MenuItemSave, "Save");
                    spawn_menu_item(parent, &assets, MenuItemQuit, "Quit");
                });
        });
}

pub fn handle_toggle_main_menu(
    action_state: Res<ActionState<Action>>,
    mut menu_layer_query: Query<&mut Visibility, With<MenuLayer>>,
    mut shell_query: Query<&mut Visibility, (With<Shell>, Without<MenuLayer>)>,
) {
    if action_state.just_pressed(Action::ToggleMainMenu) {
        let mut menu_layer_visibility = menu_layer_query.single_mut();
        let mut shell_visibility = shell_query.single_mut();

        if *menu_layer_visibility == Visibility::Inherited {
            *menu_layer_visibility = Visibility::Hidden;
            *shell_visibility = Visibility::Inherited;
        } else {
            *menu_layer_visibility = Visibility::Inherited;
            *shell_visibility = Visibility::Hidden;
        }
    }
}

pub fn handle_save(
    interaction_query: Query<&Interaction, (With<MenuItemSave>, Changed<Interaction>)>,
    mut event_writer: EventWriter<GameAction>,
) {
    if let Ok(Interaction::Clicked) = interaction_query.get_single() {
        event_writer.send(GameAction::Save());
    }
}

pub fn handle_quit(
    interaction_query: Query<&Interaction, (With<MenuItemQuit>, Changed<Interaction>)>,
    mut event_writer: EventWriter<bevy::app::AppExit>,
) {
    if let Ok(Interaction::Clicked) = interaction_query.get_single() {
        event_writer.send(bevy::app::AppExit);
    }
}
