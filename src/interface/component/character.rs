use super::super::{
    color::{NORMAL, SELECTED},
    prelude::*,
    widget::StatDisplay,
    InterfaceAssets, DEFAULT_FONT,
};
use crate::{
    actor::{Character, Members},
    creature::{Attack, Health},
    input::{SelectedIndex, Selection, SelectionUpdate},
};

fn style_title_text(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).font_size(32.0).color(css::WHITE);
}

fn style_character_list(style: &mut StyleBuilder) {
    style
        .width(Val::Px(200.0))
        .height(Val::Auto)
        .flex_direction(FlexDirection::Row)
        .align_self(AlignSelf::FlexEnd)
        .pointer_events(false);
}

fn style_character_display(style: &mut StyleBuilder) {
    style
        .width(Val::Percent(100.0))
        .height(Val::Px(60.0))
        .margin(Val::Px(2.0))
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(AlignItems::FlexStart)
        .background_color(NORMAL);
}

#[derive(Clone, PartialEq)]
pub struct CharacterList;

#[derive(Clone, PartialEq)]
pub struct CharacterDisplay {
    target: Entity,
}

#[derive(Clone, PartialEq)]
pub struct CharacterDetails {
    target: Entity,
}

impl ViewTemplate for CharacterList {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let selected = cx.use_resource::<SelectedIndex>();
        let characters: Vec<_> = selected
            .0
            .iter()
            .flat_map(|&entity| cx.use_component::<Members>(entity))
            .flat_map(|members| members.iter().cloned())
            .collect();
        Element::<NodeBundle>::new()
            .style(style_character_list)
            .children(For::each(characters, |&target| CharacterDisplay { target }))
    }
}

impl ViewTemplate for CharacterDisplay {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let target = self.target;
        let is_selected = cx
            .use_component::<Selection>(self.target)
            .unwrap()
            .is_selected;
        let on_click = cx.create_callback(
            move |character: In<Entity>, mut selection: SelectionUpdate<With<Character>>| {
                selection.toggle(*character);
            },
        );
        Element::<ButtonBundle>::new()
            .style(style_character_display)
            .style_dyn(
                move |is_selected, sb| {
                    sb.background_color(if is_selected { SELECTED } else { NORMAL });
                },
                is_selected,
            )
            .insert_dyn(
                move |_| {
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        world.run_callback(on_click, target);
                    })
                },
                (),
            )
            .children(CharacterDetails {
                target: self.target,
            })
    }
}

impl ViewTemplate for CharacterDetails {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let assets = cx.use_resource::<InterfaceAssets>();
        let character = cx.use_component::<Character>(self.target).unwrap();
        let attack = cx.use_component::<Attack>(self.target).unwrap();
        let health = cx.use_component::<Health>(self.target).unwrap();

        (
            Element::<NodeBundle>::new()
                .style(style_title_text)
                .children(character.name.clone()),
            Element::<NodeBundle>::new().children((
                StatDisplay::new(
                    assets.gladius_icon.clone(),
                    format!("{}-{}", attack.low, attack.high),
                ),
                StatDisplay::new(
                    assets.heart_shield_icon.clone(),
                    format!("{}", health.current),
                ),
            )),
        )
    }
}
