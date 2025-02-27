use super::super::{
    color::{HOVERED, NORMAL, PRESSED},
    styles::style_button,
};
use super::Tooltip;
use bevy::prelude::*;
use bevy_mod_stylebuilder::*;
use bevy_quill_core::{
    effects::{AppendEffect, EffectTuple, EntityEffect},
    prelude::*,
    style::{ApplyDynamicStylesEffect, ApplyStaticStylesEffect},
    IntoViewChild, ViewChild,
};

pub fn style_button_interaction(interaction: Interaction, style: &mut StyleBuilder) {
    style.background_color(match interaction {
        Interaction::Pressed => PRESSED,
        Interaction::Hovered => HOVERED,
        Interaction::None => NORMAL,
    });
}

#[derive(Clone, PartialEq)]
pub struct Button<E: EffectTuple = ()> {
    pub entity: Option<Entity>,
    pub children: ViewChild,
    pub on_click: Option<Callback>,
    pub style: StyleHandle,
    pub tooltip: Option<Tooltip>,
    pub icon: Option<Handle<Image>>,
    pub effects: E,
}

impl Button<()> {
    pub fn new() -> Self {
        Self {
            entity: None,
            children: ViewChild::default(),
            on_click: None,
            style: StyleHandle::default(),
            tooltip: None,
            icon: None,
            effects: (),
        }
    }
}

impl<E: EffectTuple> Button<E> {
    pub fn on_click(mut self, callback: Callback) -> Self {
        self.on_click = Some(callback);
        self
    }

    pub fn children(mut self, children: impl IntoViewChild) -> Self {
        self.children = children.into_view_child();
        self
    }

    pub fn style<S: StyleTuple + Clone + 'static>(
        self,
        styles: S,
    ) -> Button<<E as AppendEffect<ApplyStaticStylesEffect<StyleHandle>>>::Result>
    where
        E: AppendEffect<ApplyStaticStylesEffect<StyleHandle>>,
    {
        self.add_effect(ApplyStaticStylesEffect {
            styles: styles.into_handle(),
        })
    }

    pub fn style_dyn<
        S: Fn(D, &mut StyleBuilder) + Clone + Send + Sync,
        D: PartialEq + Clone + Send + Sync,
    >(
        self,
        style_fn: S,
        deps: D,
    ) -> Button<<E as AppendEffect<ApplyDynamicStylesEffect<S, D>>>::Result>
    where
        E: AppendEffect<ApplyDynamicStylesEffect<S, D>>,
    {
        self.add_effect(ApplyDynamicStylesEffect { style_fn, deps })
    }

    pub fn tooltip(mut self, tooltip: Tooltip) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub fn icon(mut self, icon: Handle<Image>) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn add_effect<E1: EntityEffect>(self, effect: E1) -> Button<<E as AppendEffect<E1>>::Result>
    where
        E: AppendEffect<E1>,
    {
        Button {
            entity: self.entity,
            children: self.children,
            on_click: self.on_click,
            style: self.style,
            tooltip: self.tooltip,
            icon: self.icon,
            effects: self.effects.append_effect(effect),
        }
    }
}

impl<E: EffectTuple + 'static> ViewTemplate for Button<E> {
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
            .style_dyn(style_button_interaction, interaction)
            .add_effect(self.effects.clone())
            .children((self.children.clone(), tooltip))
    }
}
