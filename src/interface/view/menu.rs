use super::super::{
    color::{BACKGROUND, MENU},
    prelude::*,
    styles::style_root_container,
    widget::Button,
    InterfaceState, DEFAULT_FONT,
};
use crate::{
    input::{Action, ActionState},
    scene::SceneState,
};

fn style_menu_button(style: &mut StyleBuilder) {
    style
        .width(Val::Percent(100.0))
        .height(Val::Px(50.0))
        .margin_bottom(Val::Px(10.0));
}

fn style_menu_container(style: &mut StyleBuilder) {
    style
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceAround)
        .background_color(BACKGROUND);
}

fn style_menu(style: &mut StyleBuilder) {
    style
        .width(Val::Px(300.0))
        .height(Val::Px(400.0))
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::FlexStart)
        .padding(Val::Px(5.0))
        .background_color(MENU);
}

fn style_text(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).color(css::WHITE).font_size(32.0);
}

#[derive(Clone, PartialEq)]
struct MenuItem {
    label: &'static str,
    on_click: Callback,
}

impl ViewTemplate for MenuItem {
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        Button::new()
            .on_click(self.on_click)
            .style((style_menu_button, style_text))
            .children(self.label)
    }
}

#[derive(Clone, PartialEq)]
struct Menu;

impl ViewTemplate for Menu {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        Element::<Node>::new().style(style_menu).children((
            MenuItem {
                label: "Resume",
                on_click: cx.create_callback(handle_resume),
            },
            MenuItem {
                label: "New game",
                on_click: cx.create_callback(handle_new_game),
            },
            MenuItem {
                label: "Save",
                on_click: cx.create_callback(handle_save),
            },
            MenuItem {
                label: "Quit",
                on_click: cx.create_callback(handle_quit),
            },
        ))
    }
}

#[derive(Clone, PartialEq)]
pub struct MenuView;

impl ViewTemplate for MenuView {
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        Element::<Node>::new()
            .named("Menu screen")
            .style((style_root_container, style_menu_container))
            .children(Menu)
    }
}

pub fn handle_resume(
    scene_state: Res<State<SceneState>>,
    mut next_interface_state: ResMut<NextState<InterfaceState>>,
) {
    if *scene_state.get() != SceneState::Active {
        warn!("Can't resume; No active game");
        return;
    }
    next_interface_state.set(InterfaceState::Shell);
}

pub fn handle_new_game(
    mut next_interface_state: ResMut<NextState<InterfaceState>>,
    mut next_scene_state: ResMut<NextState<SceneState>>,
) {
    next_interface_state.set(InterfaceState::Shell);
    next_scene_state.set(SceneState::Reset);
}

pub fn handle_save(mut action_state: ResMut<ActionState<Action>>) {
    action_state.press(&Action::Save);
}

pub fn handle_quit(mut event_writer: EventWriter<bevy::app::AppExit>) {
    event_writer.send(bevy::app::AppExit::Success);
}
