use super::super::{
    color::{NORMAL, SELECTED},
    prelude::*,
    resource::*,
    styles::{style_icon, style_outliner},
    widget::{Button, StatDisplay},
    InterfaceAssets, DEFAULT_FONT,
};
use crate::{
    action::ActionPoints,
    actor::{Character, Members, Party},
    input::{Selection, SelectionUpdate},
    inventory::Inventory,
};

fn style_title_text(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).font_size(32.0).color(css::WHITE);
}

#[derive(Clone, PartialEq)]
pub struct PartyList;

#[derive(Clone, PartialEq)]
pub struct PartyIcon {
    target: Entity,
}

impl PartyIcon {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

#[derive(Clone, PartialEq)]
pub struct PartyDetails {
    target: Entity,
}

impl PartyDetails {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

impl ViewTemplate for PartyList {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let party_index = cx.use_resource::<Index<Party>>();
        Element::<Node>::new()
            .style(style_outliner)
            .children(For::each(party_index.0.clone(), |&target| {
                PartyIcon::new(target)
            }))
    }
}

impl ViewTemplate for PartyIcon {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let target = self.target;
        let assets = cx.use_resource::<InterfaceAssets>();
        let icon = assets.brutal_helm_icon.clone();
        let is_selected = cx
            .use_component::<Selection>(target)
            .cloned()
            .unwrap_or_default()
            .is_selected;

        let callback =
            cx.create_callback(move |mut selection: SelectionUpdate<Without<Character>>| {
                selection.toggle(target);
            });

        Button::new()
            .icon(icon.clone())
            .on_click(callback)
            .style(style_icon)
            .style_dyn(
                |is_selected, sb| {
                    sb.border(Val::Px(4.0)).border_color(if is_selected {
                        SELECTED
                    } else {
                        NORMAL
                    });
                },
                is_selected,
            )
    }
}

impl ViewTemplate for PartyDetails {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let assets = cx.use_resource::<InterfaceAssets>();
        let party = cx.use_component::<Party>(self.target).unwrap();
        let action_points = cx.use_component::<ActionPoints>(self.target).unwrap();
        let members = cx.use_component::<Members>(self.target).unwrap();
        let inventory = cx.use_component::<Inventory>(self.target).unwrap();

        (
            Element::<Node>::new()
                .style(style_title_text)
                .children(party.name.clone()),
            Element::<Node>::new().children((
                StatDisplay::new(
                    assets.footsteps_icon.clone(),
                    format!("{}", action_points.current),
                ),
                StatDisplay::new(assets.person_icon.clone(), format!("{}", members.len())),
                StatDisplay::new(
                    assets.crystals_icon.clone(),
                    format!("{}", inventory.count_item(Inventory::CRYSTAL)),
                ),
            )),
        )
    }
}
