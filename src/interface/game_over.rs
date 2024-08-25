use super::{color::*, style::*, styles::style_root_container, InterfaceAssets, InterfaceState};
use crate::scene::{SceneState, Score};
use bevy::{color::palettes::css, prelude::*};

#[derive(Component)]
pub struct GameOverLayer;

#[derive(Component)]
pub struct NewGameButton;

fn style_game_over_layer(style: &mut StyleBuilder) {
    style
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceAround)
        .background_color(BACKGROUND);
}

fn style_game_over_display(style: &mut StyleBuilder) {
    style
        .width(Val::Px(400.0))
        .height(Val::Px(300.0))
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::FlexStart)
        .padding(Val::Px(5.0))
        .background_color(NORMAL);
}

fn style_new_game_button(style: &mut StyleBuilder) {
    style
        .width(Val::Percent(100.0))
        .height(Val::Px(50.0))
        .margin_top(Val::Auto)
        .margin_bottom(Val::Px(10.0))
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceAround)
        .background_color(MENU);
}

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
            NodeBundle::default(),
        ))
        .with_style((style_root_container, style_game_over_layer))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle::default())
                .with_style(style_game_over_display)
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
                                color: css::RED.into(),
                            },
                        ),
                    ]));
                    parent
                        .spawn((NewGameButton, ButtonBundle::default()))
                        .with_style(style_new_game_button)
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Play again",
                                TextStyle {
                                    font: assets.font.clone(),
                                    font_size: 32.0,
                                    color: css::WHITE.into(),
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
