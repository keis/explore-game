use super::super::{
    color::{NORMAL, SELECTED},
    prelude::*,
    resource::*,
    styles::{style_icon, style_outliner},
    widget::{Button, StatDisplay},
    InterfaceAssets, DEFAULT_FONT,
};
use crate::{
    actor::{Character, Members},
    input::{Selection, SelectionUpdate},
    inventory::Inventory,
    structure::Camp,
};

fn style_title_text(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).font_size(32.0).color(css::WHITE);
}

#[derive(Clone, PartialEq)]
pub struct CampList;

#[derive(Clone, PartialEq)]
pub struct CampIcon {
    target: Entity,
}

impl CampIcon {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

#[derive(Clone, PartialEq)]
pub struct CampDetails {
    target: Entity,
}

impl CampDetails {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

impl ViewTemplate for CampList {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let camp_index = cx.use_resource::<Index<Camp>>();
        Element::<Node>::new()
            .style(style_outliner)
            .children(For::each(camp_index.0.clone(), |&target| {
                CampIcon::new(target)
            }))
    }
}

impl ViewTemplate for CampIcon {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let target = self.target;
        let assets = cx.use_resource::<InterfaceAssets>();
        let icon = assets.campfire_icon.clone();
        let selection = cx
            .use_component::<Selection>(target)
            .cloned()
            .unwrap_or_default();
        let callback =
            cx.create_callback(move |mut selection: SelectionUpdate<Without<Character>>| {
                selection.toggle(target);
            });
        Button::new()
            .on_click(callback)
            .icon(icon.clone())
            .style(style_icon)
            .style_dyn(
                |selection, sb| {
                    sb.border(Val::Px(2.))
                        .border_color(if selection.is_selected {
                            SELECTED
                        } else {
                            NORMAL
                        });
                },
                selection,
            )
    }
}

impl ViewTemplate for CampDetails {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let target = self.target;
        let assets = cx.use_resource::<InterfaceAssets>();
        let person_icon = assets.person_icon.clone();
        let crystals_icon = assets.crystals_icon.clone();
        let camp = cx.use_component::<Camp>(target).unwrap();
        let members = cx.use_component::<Members>(target).unwrap();
        let inventory = cx.use_component::<Inventory>(target).unwrap();

        (
            Element::<Node>::new()
                .style(|style: &mut StyleBuilder| {
                    style
                        .width(Val::Percent(100.0))
                        .justify_content(JustifyContent::SpaceBetween);
                })
                .children((Element::<Node>::new()
                    .style(style_title_text)
                    .children(camp.name.clone()),)),
            Element::<Node>::new().children((
                StatDisplay::new(person_icon.clone(), format!("{}", members.len())),
                StatDisplay::new(
                    crystals_icon.clone(),
                    format!("{}", inventory.count_item(Inventory::CRYSTAL)),
                ),
            )),
        )
    }
}
