use super::super::{
    color::*,
    component::{CampDetails, CharacterList, PartyDetails},
    prelude::*,
    styles::{style_button, style_icon},
    widget::Opt,
    InterfaceAssets,
};
use crate::{actor::Party, input::SelectedIndex, structure::Camp};

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
        let selected_type = SelectedType::use_selected_type(cx, self.target);
        let assets = cx.use_resource::<InterfaceAssets>();

        let target = self.target;
        let focused = self.focused;
        let brutal_helm_icon = assets.brutal_helm_icon.clone();
        let campfire_icon = assets.campfire_icon.clone();

        Cond::new(
            selected_type != SelectedType::Other,
            Element::<ButtonBundle>::new()
                .insert_dyn(
                    move |_| {
                        On::<Pointer<Click>>::run(move |world: &mut World| {
                            focused.set(world, Some(target));
                        })
                    },
                    (),
                )
                .style((style_button, style_icon, move |sb: &mut StyleBuilder| {
                    match selected_type {
                        SelectedType::Party => {
                            sb.background_image(brutal_helm_icon.clone());
                        }
                        SelectedType::Camp => {
                            sb.background_image(campfire_icon.clone());
                        }
                        _ => (),
                    };
                }))
                .style_dyn(
                    move |focused, sb| {
                        sb.background_color(if focused == Some(target) {
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

        Element::<NodeBundle>::new()
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

        Element::<NodeBundle>::new()
            .named("Selected Display")
            .children((
                Element::<NodeBundle>::new()
                    .named("tab-view")
                    .style(style_selected_display)
                    .children((
                        Element::<NodeBundle>::new()
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
