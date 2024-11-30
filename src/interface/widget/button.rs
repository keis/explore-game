use super::super::{
    color::{HOVERED, NORMAL, PRESSED},
    styles::style_button,
};
use super::Tooltip;
use bevy::prelude::*;
use bevy_mod_stylebuilder::*;
use bevy_quill_core::{prelude::*, IntoViewChild, ViewChild};

#[derive(Clone, Default, PartialEq)]
pub struct Button {
    pub entity: Option<Entity>,
    pub children: ViewChild,
    pub on_click: Option<Callback>,
    pub style: StyleHandle,
    pub tooltip: Option<Tooltip>,
    pub icon: Option<Handle<Image>>,
}

impl Button {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_click(mut self, callback: Callback) -> Self {
        self.on_click = Some(callback);
        self
    }

    pub fn children(mut self, children: impl IntoViewChild) -> Self {
        self.children = children.into_view_child();
        self
    }

    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    pub fn tooltip(mut self, tooltip: Tooltip) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub fn icon(mut self, icon: Handle<Image>) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl ViewTemplate for Button {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let id = self.entity.unwrap_or_else(|| cx.create_entity());
        let interaction = cx
            .use_component::<Interaction>(id)
            .cloned()
            .unwrap_or_default();
        let icon = self.icon.clone();
        let on_click = self.on_click;
        let tooltip: ViewChild = self.tooltip.clone().map_or_else(
            || ().into_view_child(),
            |tooltip| tooltip.parent(id).into_view_child(),
        );

        cx.create_observer(
            move |_click: Trigger<Pointer<Click>>, mut commands: Commands| {
                if let Some(on_click) = on_click {
                    commands.run_callback(on_click, ());
                }
            },
            id,
            on_click,
        );

        Element::<bevy::ui::widget::Button>::for_entity(id)
            .style((style_button, self.style.clone()))
            .style_dyn(
                |icon, sb| {
                    if let Some(icon) = icon {
                        sb.background_image(icon.clone());
                    }
                },
                icon,
            )
            .style_dyn(
                |interaction, sb| {
                    sb.background_color(match interaction {
                        Interaction::Pressed => PRESSED,
                        Interaction::Hovered => HOVERED,
                        Interaction::None => NORMAL,
                    });
                },
                interaction,
            )
            .children((self.children.clone(), tooltip))
    }
}
