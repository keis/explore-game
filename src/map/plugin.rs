use super::{component::*, event::*, system::*};
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use expl_hexgrid::{layout::SquareGridLayout, HexCoord};

pub struct MapPlugin<SetupScheduleLabel, SetupSystemSet>
where
    SetupScheduleLabel: ScheduleLabel + Clone,
    SetupSystemSet: SystemSet + Clone,
{
    pub setup_schedule: SetupScheduleLabel,
    pub setup_set: SetupSystemSet,
}

impl<SetupScheduleLabel, SetupSystemSet> Plugin for MapPlugin<SetupScheduleLabel, SetupSystemSet>
where
    SetupScheduleLabel: ScheduleLabel + Clone,
    SetupSystemSet: SystemSet + Clone,
{
    fn build(&self, app: &mut App) {
        app.register_type::<Fog>()
            .register_type::<FogRevealer>()
            .register_type::<HexCoord>()
            .register_type::<MapLayout>()
            .register_type::<MapPosition>()
            .register_type::<MapPresence>()
            .register_type::<Offset>()
            .register_type::<SquareGridLayout>()
            .register_type::<ViewRadius>()
            .add_systems(
                Update,
                (
                    (update_zone_visibility, log_moves).run_if(on_event::<MapEvent>()),
                    update_terrain_visibility,
                    update_presence_fog,
                ),
            )
            .add_systems(
                self.setup_schedule.clone(),
                fluff_presence.in_set(self.setup_set.clone()),
            )
            .add_event::<MapEvent>();
    }
}
