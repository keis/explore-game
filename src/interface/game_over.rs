use super::{
    color::*, prelude::*, styles::style_root_container, widget::Button, InterfaceState,
    DEFAULT_FONT,
};
use crate::scene::{SceneState, Score};
use bevy::{color::palettes::css, prelude::*};

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

fn style_game_over_text(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).font_size(32.0).color(css::WHITE);
}

fn style_detail_text(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).font_size(24.0).color(css::WHITE);
}

fn style_detail_text_red(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).font_size(24.0).color(css::RED);
}

#[derive(Clone, PartialEq)]
pub struct GameOverScreen;

impl ViewTemplate for GameOverScreen {
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        Element::<NodeBundle>::new()
            .named("Game over screen")
            .style((style_root_container, style_game_over_layer))
            .children(GameOverDisplay)
    }
}

#[derive(Clone, PartialEq)]
pub struct GameOverDisplay;

impl ViewTemplate for GameOverDisplay {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let on_click = cx.create_callback(handle_new_game_button);
        let score = cx.use_resource::<Score>();
        Element::<NodeBundle>::new()
            .style(style_game_over_display)
            .children((
                Element::<NodeBundle>::new()
                    .style(style_game_over_text)
                    .children("Game Over"),
                Element::<NodeBundle>::new()
                    .style(style_detail_text)
                    .children(format!(
                        "Survivors: {}",
                        match score.survivors {
                            0 => String::from("None"),
                            v => format!("{}", v),
                        }
                    )),
                Element::<NodeBundle>::new()
                    .style(style_detail_text)
                    .children(format!(
                        "Dead: {}",
                        match score.dead {
                            0 => String::from("None"),
                            v => format!("{}", v),
                        }
                    )),
                Element::<NodeBundle>::new()
                    .style(style_detail_text_red)
                    .children(format!(
                        "Crystals: {}",
                        match score.dead {
                            0 => String::from("None"),
                            v => format!("{}", v),
                        }
                    )),
                Button::new()
                    .style((style_new_game_button, style_game_over_text))
                    .on_click(on_click)
                    .children("Play again"),
            ))
    }
}

pub fn handle_new_game_button(
    mut next_interface_state: ResMut<NextState<InterfaceState>>,
    mut next_scene_state: ResMut<NextState<SceneState>>,
) {
    next_interface_state.set(InterfaceState::Shell);
    next_scene_state.set(SceneState::Reset);
}
