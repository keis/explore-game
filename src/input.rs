use crate::interface::ZoneText;
use crate::map::{MapPresence, PathGuided};
use crate::GameAction;
use crate::Zone;
use bevy::prelude::*;
use bevy_mod_picking::{DefaultPickingPlugins, HoverEvent, PickingEvent, Selection};
pub use leafwing_input_manager::prelude::*;
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPickingPlugins)
            .add_plugin(InputManagerPlugin::<Action>::default())
            .add_startup_system(spawn_input_manager)
            .add_system_to_stage(CoreStage::PostUpdate, handle_picking_events);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Action {
    PanCamera,
    PanCameraMotion,
    PanCameraUp,
    PanCameraDown,
    PanCameraLeft,
    PanCameraRight,
    ZoomCamera,
    MultiSelect,
}

fn spawn_input_manager(mut commands: Commands) {
    commands.spawn_bundle(InputManagerBundle {
        action_state: ActionState::default(),
        input_map: InputMap::default()
            .insert(KeyCode::Up, Action::PanCameraUp)
            .insert(KeyCode::Down, Action::PanCameraDown)
            .insert(KeyCode::Left, Action::PanCameraLeft)
            .insert(KeyCode::Right, Action::PanCameraRight)
            .insert(KeyCode::LControl, Action::MultiSelect)
            .insert(SingleAxis::mouse_wheel_y(), Action::ZoomCamera)
            .insert(MouseButton::Right, Action::PanCamera)
            .insert(DualAxis::mouse_motion(), Action::PanCameraMotion)
            .build(),
    });
}

pub fn handle_picking_events(
    mut events: EventReader<PickingEvent>,
    zone_query: Query<&Zone>,
    presence_query: Query<(Entity, &Selection), (With<PathGuided>, With<MapPresence>)>,
    mut zone_text_query: Query<&mut Text, With<ZoneText>>,
    mut game_action_event: EventWriter<GameAction>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(e) => {
                if let Ok(zone) = zone_query.get(*e) {
                    info!("Clicked a zone: {:?}", zone);
                    for (entity, _) in presence_query.iter().filter(|(_, s)| s.selected()) {
                        game_action_event.send(GameAction::MoveTo(entity, zone.position));
                    }
                } else {
                    info!("Clicked something: {:?}", e);
                }
            }
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => {
                if let Ok(zone) = zone_query.get(*e) {
                    for mut text in &mut zone_text_query {
                        text.sections[0].value = format!("{:?}", zone.position);
                    }
                }
            }
            PickingEvent::Selection(e) => {
                info!("Selection event {:?}", e);
            }
            _ => {}
        }
    }
}
