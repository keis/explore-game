use super::super::{
    color::{HOVERED, NORMAL, PRESSED},
    styles::style_button,
};
use bevy::prelude::*;
//use bevy_mod_picking::prelude::*;
use bevy_mod_stylebuilder::*;
use bevy_quill_core::{prelude::*, IntoViewChild, ViewChild};

#[derive(Clone, Default, PartialEq)]
pub struct Button {
    pub children: ViewChild,
    pub on_click: Option<Callback>,
    pub style: StyleHandle,
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
}

impl ViewTemplate for Button {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let id = cx.create_entity();
        let interaction = cx
            .use_component::<Interaction>(id)
            .cloned()
            .unwrap_or_default();
        let on_click = self.on_click;

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
                |interaction, sb| {
                    sb.background_color(match interaction {
                        Interaction::Pressed => PRESSED,
                        Interaction::Hovered => HOVERED,
                        Interaction::None => NORMAL,
                    });
                },
                interaction,
            )
            .children(self.children.clone())
    }
}
