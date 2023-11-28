use super::{color::*, InterfaceAssets, InterfaceState};
use crate::scene::{SceneState, Score};
use bevy::prelude::*;

#[derive(Component)]
pub struct GameOverLayer;

#[derive(Component)]
pub struct NewGameButton;

pub fn spawn_game_over_screen(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    score_query: Query<&Score>,
) {
    let score = score_query.get_single().unwrap();
    commands
        .spawn((
            Name::new("Game over layer"),
            GameOverLayer,
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
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(400.0),
                        height: Val::Px(300.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    background_color: NORMAL.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Game Over",
                        TextStyle {
                            font: assets.font.clone(),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ));
                    parent.spawn(TextBundle::from_sections([
                        TextSection::new(
                            "Survivors: ",
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            match score.survivors {
                                0 => String::from("None"),
                                v => format!("{}", v),
                            },
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE,
                            },
                        ),
                    ]));
                    parent.spawn(TextBundle::from_sections([
                        TextSection::new(
                            "Dead: ",
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            match score.dead {
                                0 => String::from("None"),
                                v => format!("{}", v),
                            },
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE,
                            },
                        ),
                    ]));
                    parent.spawn(TextBundle::from_sections([
                        TextSection::new(
                            "Crystals: ",
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            format!("{}", score.crystals),
                            TextStyle {
                                font: assets.font.clone(),
                                font_size: 24.0,
                                color: Color::RED,
                            },
                        ),
                    ]));
                    parent
                        .spawn((
                            NewGameButton,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(50.0),
                                    margin: UiRect {
                                        top: Val::Auto,
                                        bottom: Val::Px(10.0),
                                        ..default()
                                    },
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceAround,
                                    ..default()
                                },
                                background_color: MENU.into(),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Play again",
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });
        });
}

pub fn despawn_game_over_screen(
    mut commands: Commands,
    game_over_screen_query: Query<Entity, With<GameOverLayer>>,
) {
    if let Ok(game_over_screen_entity) = game_over_screen_query.get_single() {
        commands.entity(game_over_screen_entity).despawn_recursive();
    }
}

pub fn handle_new_game(
    interaction_query: Query<&Interaction, (With<NewGameButton>, Changed<Interaction>)>,
    mut next_interface_state: ResMut<NextState<InterfaceState>>,
    mut next_scene_state: ResMut<NextState<SceneState>>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.get_single() {
        next_interface_state.set(InterfaceState::Shell);
        next_scene_state.set(SceneState::Reset);
    }
}
