use super::super::{
    color::{NORMAL, SELECTED},
    prelude::*,
    resource::*,
    styles::{style_button, style_icon, style_outliner},
    widget::StatDisplay,
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
        let id = cx.create_entity();
        let target = self.target;
        let assets = cx.use_resource::<InterfaceAssets>();
        let icon = assets.campfire_icon.clone();
        let selection = cx
            .use_component::<Selection>(target)
            .cloned()
            .unwrap_or_default();
        let on_click = cx.create_callback(
            move |camp: In<Entity>, mut selection: SelectionUpdate<Without<Character>>| {
                selection.toggle(*camp);
            },
        );
        cx.create_observer(
            move |_click: Trigger<Pointer<Click>>, mut commands: Commands| {
                commands.run_callback(on_click, target);
            },
            id,
            target,
        );
        Element::<Button>::for_entity(id)
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
    }
}

impl ViewTemplate for CampDetails {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let assets = cx.use_resource::<InterfaceAssets>();
        let camp = cx.use_component::<Camp>(self.target).unwrap();
        let members = cx.use_component::<Members>(self.target).unwrap();
        let inventory = cx.use_component::<Inventory>(self.target).unwrap();

        (
            Element::<Node>::new()
                .style(style_title_text)
                .children(camp.name.clone()),
            Element::<Node>::new().children((
                StatDisplay::new(assets.person_icon.clone(), format!("{}", members.len())),
                StatDisplay::new(
                    assets.crystals_icon.clone(),
                    format!("{}", inventory.count_item(Inventory::CRYSTAL)),
                ),
            )),
        )
    }
}
