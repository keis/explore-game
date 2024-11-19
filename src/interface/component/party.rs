use super::super::{
    color::{NORMAL, SELECTED},
    prelude::*,
    resource::*,
    styles::{style_button, style_icon, style_outliner},
    widget::StatDisplay,
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
        Element::<NodeBundle>::new()
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
        let selection = cx
            .use_component::<Selection>(target)
            .cloned()
            .unwrap_or_default();
        let on_click = cx.create_callback(
            move |party: In<Entity>, mut selection: SelectionUpdate<Without<Character>>| {
                selection.toggle(*party);
            },
        );
        Element::<ButtonBundle>::new()
            .style((style_button, style_icon, move |sb: &mut StyleBuilder| {
                sb.background_image(icon.clone());
            }))
            .style_dyn(
                |selection, sb| {
                    sb.background_color(if selection.is_selected {
                        SELECTED
                    } else {
                        NORMAL
                    });
                },
                selection,
            )
            .insert_dyn(
                move |_| {
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        world.run_callback(on_click, target);
                    })
                },
                (),
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
            Element::<NodeBundle>::new()
                .style(style_title_text)
                .children(party.name.clone()),
            Element::<NodeBundle>::new().children((
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
