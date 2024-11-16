#![allow(clippy::type_complexity)]

mod action;
mod bundle;
mod component;
mod event;
mod plugin;
mod resource;
mod system;
mod system_param;

pub use action::Action;
pub use bundle::*;
pub use component::*;
pub use event::*;
pub use leafwing_input_manager::{
    common_conditions::{action_just_pressed, action_toggle_active},
    input_map::InputMap,
    plugin::InputManagerSystem,
    prelude::ActionState,
};
pub use plugin::{InputPlugin, InputSet};
pub use resource::*;
pub use system_param::*;

#[cfg(test)]
mod tests {
    use super::{action::*, system::*, Action, ActionState, Deselect, Select, Selection};
    use crate::{action::ActionPoints, camera::CameraControl, test_fixture::spawn_game_map};
    use bevy::prelude::*;
    use expl_map::MapPresence;
    use rstest::*;

    #[fixture]
    pub fn app() -> App {
        let mut app = App::new();
        spawn_game_map(&mut app);
        app.insert_resource(ActionState::<Action>::default());
        app.add_event::<Select>();
        app.add_event::<Deselect>();
        app.world_mut().spawn(CameraControl::default());
        app.world_mut().spawn((
            MapPresence {
                position: (1, 1).into(),
            },
            Selection::default(),
            ActionPoints {
                current: 2,
                reset: 2,
            },
        ));
        app.world_mut().spawn((
            MapPresence {
                position: (2, 0).into(),
            },
            Selection::default(),
            ActionPoints {
                current: 2,
                reset: 2,
            },
        ));
        app
    }

    pub fn get_selected_entities(app: &mut App) -> Vec<Entity> {
        app.world_mut()
            .query::<(Entity, &Selection)>()
            .iter(app.world())
            .filter(|(_, s)| s.is_selected)
            .map(|(e, _)| e)
            .collect()
    }

    pub fn press_select_next(app: &mut App) {
        app.world_mut()
            .resource_mut::<ActionState<Action>>()
            .press(&Action::SelectNext);
    }

    #[rstest]
    pub fn select_next(mut app: App) {
        app.observe(apply_select_event)
            .observe(apply_deselect_event)
            .add_systems(Update, handle_select_next);

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
