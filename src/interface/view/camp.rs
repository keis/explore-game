use super::super::{
    color, component::CharacterDetails, prelude::*, styles::style_button,
    widget::button::style_button_interaction, ShellState,
};
use crate::{
    action::{CampActionAssignment, GameActionType},
    actor::Members,
    structure::Camp,
};

fn style_center_container(style: &mut StyleBuilder) {
    style
        .margin(Val::Px(80.))
        .width(Val::Percent(100.))
        .flex_direction(FlexDirection::Column)
        .background_color(color::BACKGROUND);
}

fn style_name(style: &mut StyleBuilder) {
    style.font_size(24.0);
}

fn style_grid(style: &mut StyleBuilder) {
    style
        .display(Display::Grid)
        .grid_template_columns(RepeatedGridTrack::fr(4, 1.));
}

#[derive(Clone, PartialEq)]
pub struct CampActionButton {
    target: Entity,
    action: GameActionType,
}

impl ViewTemplate for CampActionButton {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let id = cx.create_entity();
        let interaction = cx
            .use_component::<Interaction>(id)
            .cloned()
            .unwrap_or_default();
        let assigned_camp_action = cx
            .use_component::<CampActionAssignment>(self.target)
            .cloned()
            .unwrap_or_default();
        let on_click = cx.create_callback(|| {
            info!("HELLO");
        });

        Element::<ButtonBundle>::for_entity(id)
            .style(style_button)
            .style_dyn(style_button_interaction, interaction)
            .style_dyn(
                |assigned_camp_action, style: &mut StyleBuilder| {
                    info!("STYLE {:?}", assigned_camp_action);
                },
                assigned_camp_action,
            )
            .insert_dyn(
                move |_| {
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        world.run_callback(on_click, ());
                    })
                },
                (),
            )
            .children("Korv")
    }
}

#[derive(Clone, PartialEq)]
pub struct CampMemberView {
    target: Entity,
}

impl ViewTemplate for CampMemberView {
    type View = impl View;

    fn create(&self, _cx: &mut Cx) -> Self::View {
        (
            CharacterDetails::new(self.target),
            CampActionButton {
                target: self.target,
                action: GameActionType::ManageCamp,
            },
            CampActionButton {
                target: self.target,
                action: GameActionType::GuardCamp,
            },
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct CampView;

impl ViewTemplate for CampView {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let shell_state = cx.use_resource::<State<ShellState>>();
        let &ShellState::Camp { target } = shell_state.get() else {
            panic!("Unexpected shell state");
        };
        let camp = cx.use_component::<Camp>(target).unwrap();
        let members = cx.use_component::<Members>(target).unwrap();
        Element::<NodeBundle>::new()
            .named("Camp view")
            .style(style_center_container)
            .children((
                Element::<NodeBundle>::new()
                    .style(style_name)
                    .children(camp.name.clone()),
                Element::<NodeBundle>::new()
                    .style(style_grid)
                    .children(For::each(members.0.clone(), |&target| CampMemberView {
                        target,
                    })),
            ))
    }
}
