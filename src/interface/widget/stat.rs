use super::super::{prelude::*, styles::style_small_icon, DEFAULT_FONT};

fn style_stat_display(style: &mut StyleBuilder) {
    style.padding(Val::Px(2.0)).align_items(AlignItems::Center);
}

fn style_stat_display_text(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).font_size(24.0).color(css::WHITE);
}

#[derive(Clone, PartialEq)]
pub struct StatDisplay {
    pub icon: Handle<Image>,
    pub children: ViewChild,
}

impl StatDisplay {
    pub fn new(icon: Handle<Image>, children: impl IntoViewChild) -> Self {
        Self {
            icon,
            children: children.into_view_child(),
        }
    }
}

impl ViewTemplate for StatDisplay {
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        Element::<NodeBundle>::new()
            .style((style_stat_display, style_stat_display_text))
            .children((
                Element::<ImageBundle>::new()
                    .style(style_small_icon)
                    .insert(UiImage::from(self.icon.clone())),
                self.children.clone(),
            ))
    }
}
