use super::super::{
    color::*,
    component::{CampDetails, CharacterList, PartyDetails},
    prelude::*,
    styles::style_icon,
    widget::{Button, Opt},
    InterfaceAssets,
};
use crate::{actor::Party, input::SelectedIndex, structure::Camp};
use bevy::ecs::world::DeferredWorld;

fn style_selected_display(style: &mut StyleBuilder) {
    style
        .width(Val::Px(200.0))
        .flex_direction(FlexDirection::Column);
}

fn style_selected_item(style: &mut StyleBuilder) {
    style
        .width(Val::Percent(100.0))
        .height(Val::Px(120.0))
        .margin(Val::Px(2.0))
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(AlignItems::FlexStart)
        .background_color(NORMAL);
}

#[derive(Clone, PartialEq, Debug)]
pub enum SelectedType {
    Party,
    Camp,
    Other,
}

impl SelectedType {
    fn use_selected_type(cx: &mut Cx, entity: Entity) -> Self {
        if cx.use_component::<Party>(entity).is_some() {
            SelectedType::Party
        } else if cx.use_component::<Camp>(entity).is_some() {
            SelectedType::Camp
        } else {
            SelectedType::Other
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SelectedView;

#[derive(Clone, PartialEq)]
pub struct SelectedTabHeaderIcon {
    target: Entity,
    focused: Mutable<Option<Entity>>,
}

impl SelectedTabHeaderIcon {
    pub fn new(target: Entity, focused: Mutable<Option<Entity>>) -> Self {
        Self { target, focused }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedTabViewContent {
    target: Entity,
}

impl SelectedTabViewContent {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

impl ViewTemplate for SelectedTabHeaderIcon {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let id = cx.create_entity();
        let selected_type = SelectedType::use_selected_type(cx, self.target);
        let assets = cx.use_resource::<InterfaceAssets>();

        let target = self.target;
        let focused = self.focused;
        let brutal_helm_icon = assets.brutal_helm_icon.clone();
        let campfire_icon = assets.campfire_icon.clone();

        cx.create_observer(
            move |_click: Trigger<Pointer<Click>>, mut world: DeferredWorld| {
                focused.set(&mut world, Some(target));
            },
            id,
            target,
        );

        let callback = cx.create_callback(move |mut world: DeferredWorld| {
            focused.set(&mut world, Some(target));
        });

        Cond::new(
            selected_type != SelectedType::Other,
            Button::new()
                .on_click(callback)
                .icon(match selected_type {
                    SelectedType::Party => brutal_helm_icon.clone(),
                    SelectedType::Camp => campfire_icon.clone(),
                    _ => brutal_helm_icon.clone(),
                })
                .style(style_icon)
                .style_dyn(
                    move |focused, sb| {
                        sb.border(Val::Px(2.0))
                            .border_color(if focused == Some(target) {
                                SELECTED
                            } else {
                                NORMAL
                            });
                    },
                    focused.get(cx),
                ),
            (),
        )
    }
}

impl ViewTemplate for SelectedTabViewContent {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let selected_type = SelectedType::use_selected_type(cx, self.target);
        let target = self.target;

        Element::<Node>::new()
            .named("Selected details")
            .style(style_selected_item)
            .children(
                Switch::new(selected_type)
                    .case(SelectedType::Party, PartyDetails::new(target))
                    .case(SelectedType::Camp, CampDetails::new(target)),
            )
    }
}

impl ViewTemplate for SelectedView {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let mut selected = cx.use_resource::<SelectedIndex>().0.clone();
        selected
            .retain(|&entity| SelectedType::use_selected_type(cx, entity) != SelectedType::Other);
        let focused = cx.create_mutable::<Option<Entity>>(None);

        focused.update(cx, |mut focused_entity: Mut<Option<Entity>>| {
            let new_focus = focused_entity
                .filter(|entity| selected.contains(entity))
                .or(selected.first().copied());
            if new_focus != *focused_entity {
                *focused_entity = new_focus;
            }
        });

        let focused_entity = focused.get(cx);

        Element::<Node>::new().named("Selected Display").children((
            Element::<Node>::new()
                .named("tab-view")
                .style(style_selected_display)
                .children((
                    Element::<Node>::new()
                        .named("tab-header")
                        .children(For::each(selected, move |&target| {
                            SelectedTabHeaderIcon::new(target, focused)
                        })),
                    Opt::new(focused_entity.map(SelectedTabViewContent::new)),
                )),
            CharacterList,
        ))
    }
}
