use super::super::{color::NORMAL, prelude::*};

#[derive(Copy, Clone, Default, PartialEq)]
pub enum TooltipPosition {
    #[default]
    Above,
    Below,
}

fn style_tooltip(style: &mut StyleBuilder) {
    style
        .position(PositionType::Absolute)
        .padding(Val::Px(4.0))
        .background_color(NORMAL);
}

#[derive(Clone, PartialEq)]
pub struct Tooltip {
    parent: Entity,
    position: TooltipPosition,
    children: ViewChild,
}

impl Tooltip {
    pub fn for_parent(parent: Entity) -> Self {
        Self {
            parent,
            position: TooltipPosition::default(),
            children: ViewChild::default(),
        }
    }

    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    pub fn children(mut self, children: impl IntoViewChild) -> Self {
        self.children = children.into_view_child();
        self
    }
}

impl ViewTemplate for Tooltip {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let interaction = cx
            .use_component::<Interaction>(self.parent)
            .cloned()
            .unwrap_or_default();
        Cond::new(
            interaction == Interaction::Hovered,
            Element::<NodeBundle>::new()
                .named("Tooltip")
                .style(style_tooltip)
                .style_dyn(
                    |position, sb| {
                        match position {
                            TooltipPosition::Above => sb.bottom(Val::Px(60.0)),
                            TooltipPosition::Below => sb.top(Val::Px(40.0)),
                        };
                    },
                    self.position,
                )
                .children(self.children.clone()),
            (),
        )
    }
}