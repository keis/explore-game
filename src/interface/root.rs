use super::{
    plugin::InterfaceState,
    prelude::*,
    view::{GameOverView, MenuView, ShellView},
};

#[derive(Clone, PartialEq)]
struct Interface;

impl ViewTemplate for Interface {
    type View = impl View;

    fn create(&self, cx: &mut Cx) -> Self::View {
        let interface_state = cx.use_resource::<State<InterfaceState>>();
        Switch::new(*interface_state.get())
            .case(InterfaceState::Hidden, ())
            .case(InterfaceState::Shell, ShellView)
            .case(InterfaceState::Menu, MenuView)
            .case(InterfaceState::GameOver, GameOverView)
    }
}

pub fn spawn_interface_root(mut commands: Commands) {
    commands.spawn((Name::new("Interface Root"), Interface.to_root()));
}
