use super::super::{
    color, component::CharacterDetails, prelude::*, styles::style_button, widget::Button,
    ShellState,
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
        let target = self.target;
        let action = self.action;
        let assigned_camp_action = cx
            .use_component::<CampActionAssignment>(target)
            .cloned()
            .unwrap_or_default();
        let callback = cx.create_callback(
            move |mut assigned_camp_action_query: Query<&mut CampActionAssignment>| {
                if let Ok(mut assigned_camp_action) = assigned_camp_action_query.get_mut(target) {
                    assigned_camp_action.action_type = action;
                }
            },
        );

        Button::new()
            .on_click(callback)
            .style_dyn(
                move |assigned_camp_action, style: &mut StyleBuilder| {
                    style.border(Val::Px(2.)).border_color(
                        if assigned_camp_action.action_type == action {
                            color::SELECTED
                        } else {
                            color::NORMAL
                        },
                    );
                },
                assigned_camp_action,
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
        Element::<Node>::new()
            .named("Camp view")
            .style(style_center_container)
            .children((
                Element::<Node>::new()
                    .style(style_name)
                    .children(camp.name.clone()),
                Element::<Node>::new()
                    .style(style_grid)
                    .children(For::each(members.0.clone(), |&target| CampMemberView {
                        target,
                    })),
            ))
    }
}
