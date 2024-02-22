use super::{
    color::{BACKGROUND, HOVERED, MENU, NORMAL, PRESSED},
    InterfaceAssets, InterfaceState,
};
use crate::{
    input::{Action, ActionState},
    scene::SceneState,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct MenuLayer;

#[derive(Component)]
pub struct MenuItem;

#[derive(Component)]
pub struct MenuItemResume;

#[derive(Component)]
pub struct MenuItemNewGame;

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
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
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
            MenuItem,
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
            Name::new("Menu Container"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
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
                        width: Val::Px(300.0),
                        height: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: MENU.into(),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_menu_item(parent, &assets, MenuItemResume, "Resume");
                    spawn_menu_item(parent, &assets, MenuItemNewGame, "New game");
                    spawn_menu_item(parent, &assets, MenuItemSave, "Save");
                    spawn_menu_item(parent, &assets, MenuItemQuit, "Quit");
                });
        });
}

pub fn show_menu(mut menu_layer_query: Query<&mut Visibility, With<MenuLayer>>) {
    let mut menu_layer_visibility = menu_layer_query.single_mut();
    *menu_layer_visibility = Visibility::Inherited;
}

pub fn hide_menu(mut menu_layer_query: Query<&mut Visibility, With<MenuLayer>>) {
    let mut menu_layer_visibility = menu_layer_query.single_mut();
    *menu_layer_visibility = Visibility::Hidden;
}

pub fn handle_toggle_main_menu(
    current_state: Res<State<InterfaceState>>,
    mut next_state: ResMut<NextState<InterfaceState>>,
) {
    next_state.set(match current_state.get() {
        InterfaceState::Hidden => InterfaceState::Menu,
        InterfaceState::Menu => InterfaceState::Shell,
        InterfaceState::Shell => InterfaceState::Menu,
        InterfaceState::GameOver => InterfaceState::GameOver,
    });
}

#[allow(clippy::type_complexity)]
pub fn menu_item_interaction_effect(
    mut menu_item_query: Query<
        (&mut BackgroundColor, &Interaction),
        (With<MenuItem>, Changed<Interaction>),
    >,
) {
    for (mut background_color, interaction) in &mut menu_item_query {
        *background_color = match interaction {
            Interaction::Pressed => PRESSED.into(),
            Interaction::Hovered => HOVERED.into(),
            Interaction::None => NORMAL.into(),
        }
    }
}

pub fn handle_resume(
    interaction_query: Query<&Interaction, (With<MenuItemResume>, Changed<Interaction>)>,
    scene_state: Res<State<SceneState>>,
    mut next_interface_state: ResMut<NextState<InterfaceState>>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.get_single() {
        if *scene_state.get() != SceneState::Active {
            warn!("Can't resume; No active game");
            return;
        }
        next_interface_state.set(InterfaceState::Shell);
    }
}

pub fn handle_new_game(
    interaction_query: Query<&Interaction, (With<MenuItemNewGame>, Changed<Interaction>)>,
    mut next_interface_state: ResMut<NextState<InterfaceState>>,
    mut next_scene_state: ResMut<NextState<SceneState>>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.get_single() {
        next_interface_state.set(InterfaceState::Shell);
        next_scene_state.set(SceneState::Reset);
    }
}

pub fn handle_save(
    interaction_query: Query<&Interaction, (With<MenuItemSave>, Changed<Interaction>)>,
    mut action_state: ResMut<ActionState<Action>>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.get_single() {
        action_state.press(&Action::Save);
    }
}

pub fn handle_quit(
    interaction_query: Query<&Interaction, (With<MenuItemQuit>, Changed<Interaction>)>,
    mut event_writer: EventWriter<bevy::app::AppExit>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.get_single() {
        event_writer.send(bevy::app::AppExit);
    }
}
