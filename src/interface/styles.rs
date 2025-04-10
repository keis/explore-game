use super::{color, prelude::*};

pub fn style_root_container(style: &mut StyleBuilder) {
    style.width(Val::Percent(100.0)).height(Val::Percent(100.0));
}

pub fn style_button(style: &mut StyleBuilder) {
    style
        .background_color(color::NORMAL)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceAround)
        .margin(Val::Px(2.0));
}

pub fn style_icon(style: &mut StyleBuilder) {
    style.width(Val::Px(32.0)).height(Val::Px(32.0));
}

pub fn style_small_icon(style: &mut StyleBuilder) {
    style.width(Val::Px(20.0)).height(Val::Px(20.0));
}

pub fn style_outliner(style: &mut StyleBuilder) {
    style
        .flex_direction(FlexDirection::Column)
        .pointer_events(false);
}
