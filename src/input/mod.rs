#![allow(clippy::type_complexity)]

mod action;
mod bundle;
mod component;
mod event;
mod plugin;
mod system;
mod system_param;

pub use action::Action;
pub use bundle::*;
pub use component::*;
pub use event::*;
pub use leafwing_input_manager::{
    common_conditions::{action_just_pressed, action_toggle_active},
    plugin::InputManagerSystem,
    prelude::ActionState,
};
pub use plugin::InputPlugin;
pub use system_param::*;

#[cfg(test)]
mod tests {
    use super::{action::handle_select_next, Action, ActionState, Selection};
    use crate::{
        actor::Movement, camera::CameraControl, map::MapPresence, test_fixture::spawn_game_map,
    };
    use bevy::prelude::*;
    use rstest::*;

    #[fixture]
    pub fn app() -> App {
        let mut app = App::new();
        spawn_game_map(&mut app);
        app.insert_resource(ActionState::<Action>::default());
        app.world.spawn(CameraControl::default());
        app.world.spawn((
            MapPresence {
                position: (1, 1).into(),
            },
            Selection::default(),
            Movement {
                current: 2,
                reset: 2,
            },
        ));
        app.world.spawn((
            MapPresence {
                position: (2, 0).into(),
            },
            Selection::default(),
            Movement {
                current: 2,
                reset: 2,
            },
        ));
        app
    }

    pub fn get_selected_entities(app: &mut App) -> Vec<Entity> {
        app.world
            .query::<(Entity, &Selection)>()
            .iter(&app.world)
            .filter(|(_, s)| s.is_selected)
            .map(|(e, _)| e)
            .collect()
    }

    pub fn press_select_next(app: &mut App) {
        app.world
            .resource_mut::<ActionState<Action>>()
            .press(Action::SelectNext);
    }

    #[rstest]
    pub fn select_next(mut app: App) {
        app.add_systems(Update, handle_select_next);

        press_select_next(&mut app);
        app.update();

        let selected1 = get_selected_entities(&mut app);

        press_select_next(&mut app);
        app.update();

        let selected2 = get_selected_entities(&mut app);

        press_select_next(&mut app);
        app.update();

        let selected3 = get_selected_entities(&mut app);

        assert_eq!(selected1.len(), 1);
        assert_eq!(selected2.len(), 1);
        assert_eq!(selected3.len(), 1);
        assert_ne!(selected1[0], selected2[0]);
        assert_eq!(selected1[0], selected3[0]);
    }
}
