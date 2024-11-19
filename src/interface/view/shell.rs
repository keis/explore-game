use super::super::{
    color::*,
    component::{CampList, PartyList},
    prelude::*,
    styles::{style_button, style_icon, style_outliner, style_root_container},
    widget::{Opt, Tooltip, TooltipPosition},
    InterfaceAssets, DEFAULT_FONT,
};
use super::SelectedView;
use crate::{
    input::{Action, ActionState, InputMap, MapHover},
    turn::Turn,
};
use expl_map::MapPosition;

fn style_toolbar(style: &mut StyleBuilder) {
    style.align_self(AlignSelf::FlexStart).padding(Val::Px(2.0));
}

fn style_tooltip_text(style: &mut StyleBuilder) {
    style
        .margin(Val::Px(2.0))
        .font(DEFAULT_FONT)
        .font_size(20.0)
        .color(css::WHITE);
}

fn style_keybind_text(style: &mut StyleBuilder) {
    style
        .margin(Val::Px(2.0))
        .font(DEFAULT_FONT)
        .font_size(20.0)
        .color(css::GREEN);
}

fn style_shell_container(style: &mut StyleBuilder) {
    style
        .flex_direction(FlexDirection::Column)
        .justify_content(JustifyContent::SpaceBetween)
        .pointer_events(false);
}

fn style_bar(style: &mut StyleBuilder) {
    style
        .width(Val::Percent(100.0))
        .justify_content(JustifyContent::SpaceBetween)
        .pointer_events(false);
}

fn style_zone_display(style: &mut StyleBuilder) {
    style
        .width(Val::Px(100.0))
        .justify_content(JustifyContent::End);
}

fn style_zone_display_text(style: &mut StyleBuilder) {
    style.font(DEFAULT_FONT).font_size(32.0).color(css::WHITE);
}

fn style_next_turn_button(style: &mut StyleBuilder) {
    style
        .width(Val::Px(200.0))
        .height(Val::Px(60.0))
        .align_self(AlignSelf::FlexEnd)
        .background_color(HIGHLIGHT)
        .font_size(32.0)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center);
}

fn get_keybind_for_action(inputmap: &InputMap<Action>, action: &Action) -> Option<KeyCode> {
    inputmap
        .get_buttonlike(action)
        .and_then(|v| v.first())
        .and_then(|buttonlike| Reflect::as_any(buttonlike.as_ref()).downcast_ref::<KeyCode>())
        .copied()
}

#[derive(Clone, PartialEq)]
pub struct KeybindText {
    keybind: KeyCode,
}

impl KeybindText {
    fn new(keybind: KeyCode) -> Self {
        Self { keybind }
    }

    fn keybind_text(&self) -> &'static str {
        match self.keybind {
            KeyCode::KeyC => "<C>",
            KeyCode::KeyM => "<M>",
            KeyCode::Enter => "<Enter>",
            _ => "-",
        }
    }
}

impl ViewTemplate for KeybindText {
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        Element::<NodeBundle>::new()
            .style(style_keybind_text)
            .children(self.keybind_text())
    }
}

#[derive(Clone, PartialEq)]
pub struct TooltipContent {
    tooltip_text: String,
    keybind: Option<KeyCode>,
}

impl TooltipContent {
    fn new(tooltip_text: impl Into<String>) -> Self {
        TooltipContent {
            tooltip_text: tooltip_text.into(),
            keybind: None,
        }
    }

    fn maybe_keybind(mut self, keybind: Option<KeyCode>) -> Self {
        self.keybind = keybind;
        self
    }
}

impl ViewTemplate for TooltipContent {
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        Element::<NodeBundle>::new()
            .style(style_tooltip_text)
            .children((
                self.tooltip_text.clone(),
                Opt::new(self.keybind.map(KeybindText::new)),
            ))
    }
}

#[derive(Clone, PartialEq)]
struct ToolbarItem {
    action: Action,
    icon: Handle<Image>,
    tooltip_text: String,
}

impl ToolbarItem {
    pub fn for_action(action: Action) -> Self {
        Self {
            action,
            icon: Handle::default(),
            tooltip_text: "".to_string(),
        }
    }

    pub fn icon(mut self, icon: Handle<Image>) -> Self {
        self.icon = icon;
        self
    }

    pub fn tooltip_text(mut self, tooltip_text: impl Into<String>) -> Self {
        self.tooltip_text = tooltip_text.into();
        self
    }
}

impl ViewTemplate for ToolbarItem {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let id = cx.create_entity();
        let icon = self.icon.clone();
        let action = self.action;
        let inputmap = cx.use_resource::<InputMap<Action>>();
        let keybind = get_keybind_for_action(inputmap, &action);
        Element::<ButtonBundle>::for_entity(id)
            .insert_dyn(
                move |_| {
                    On::<Pointer<Click>>::run(
                        move |mut action_state: ResMut<ActionState<Action>>| {
                            action_state.press(&action);
                        },
                    )
                },
                (),
            )
            .style((style_button, style_icon, move |sb: &mut StyleBuilder| {
                sb.background_image(icon.clone());
            }))
            .children(
                Tooltip::for_parent(id)
                    .position(TooltipPosition::Below)
                    .children(
                        TooltipContent::new(self.tooltip_text.clone()).maybe_keybind(keybind),
                    ),
            )
    }
}

#[derive(Clone, PartialEq)]
struct Toolbar;

impl ViewTemplate for Toolbar {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let assets = cx.use_resource::<InterfaceAssets>();
        Element::<NodeBundle>::new().style(style_toolbar).children((
            ToolbarItem::for_action(Action::ResumeMove)
                .icon(assets.arrow_icon.clone())
                .tooltip_text("Resume move"),
            ToolbarItem::for_action(Action::Camp)
                .icon(assets.campfire_icon.clone())
                .tooltip_text("Make/Enter camp"),
            ToolbarItem::for_action(Action::BreakCamp)
                .icon(assets.cancel_icon.clone())
                .tooltip_text("Break camp"),
            ToolbarItem::for_action(Action::CreateParty)
                .icon(assets.knapsack_icon.clone())
                .tooltip_text("Create party"),
            ToolbarItem::for_action(Action::SplitParty)
                .icon(assets.back_forth_icon.clone())
                .tooltip_text("Split selected from party"),
            ToolbarItem::for_action(Action::MergeParty)
                .icon(assets.contract_icon.clone())
                .tooltip_text("Merge selected parties"),
            ToolbarItem::for_action(Action::CollectCrystals)
                .icon(assets.crystals_icon.clone())
                .tooltip_text("Collect cyrstals"),
            ToolbarItem::for_action(Action::OpenPortal)
                .icon(assets.magic_swirl_icon.clone())
                .tooltip_text("Open portal"),
            ToolbarItem::for_action(Action::EnterPortal)
                .icon(assets.portal_icon.clone())
                .tooltip_text("Enter portal"),
        ))
    }
}

#[derive(Clone, PartialEq)]
struct ZoneDisplay;

impl ViewTemplate for ZoneDisplay {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let map_hover = cx.use_resource::<MapHover>();
        let text = if let Some(map_position) = map_hover
            .zone
            .and_then(|e| cx.use_component_untracked::<MapPosition>(e))
        {
            format!("Zone: {}", map_position.0)
        } else {
            String::from("")
        };
        Element::<NodeBundle>::new()
            .named("Zone Display")
            .style((style_zone_display, style_zone_display_text))
            .children(text)
    }
}

#[derive(Clone, PartialEq)]
pub struct NextTurnButton;

impl ViewTemplate for NextTurnButton {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let id = cx.create_entity();
        let turn = cx.use_resource::<Turn>();
        let action = Action::NextTurn;
        let inputmap = cx.use_resource::<InputMap<Action>>();
        let keybind = get_keybind_for_action(inputmap, &action);
        Element::<ButtonBundle>::for_entity(id)
            .named("Next Turn Button")
            .style(style_next_turn_button)
            .insert_dyn(
                move |_| {
                    On::<Pointer<Click>>::run(
                        move |mut action_state: ResMut<ActionState<Action>>| {
                            action_state.press(&action);
                        },
                    )
                },
                (),
            )
            .children((
                format!("Turn {}", **turn),
                Tooltip::for_parent(id)
                    .children(TooltipContent::new("Next turn").maybe_keybind(keybind)),
            ))
    }
}

#[derive(Clone, PartialEq)]
pub struct ShellView;

impl ViewTemplate for ShellView {
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        Element::<NodeBundle>::new()
            .named("Shell screen")
            .style((style_root_container, style_shell_container))
            .children((
                Element::<NodeBundle>::new()
                    .named("Top")
                    .style(style_bar)
                    .children((
                        Element::<NodeBundle>::new()
                            .named("Outliner")
                            .style(style_outliner)
                            .children((CampList, PartyList)),
                        Toolbar,
                        ZoneDisplay,
                    )),
                Element::<NodeBundle>::new()
                    .named("Bottom")
                    .style(style_bar)
                    .children((SelectedView, NextTurnButton)),
            ))
    }
}
