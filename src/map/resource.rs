use super::event::MapEvent;
use bevy::prelude::*;

#[derive(Resource)]
pub struct Damaged(pub bool);

#[derive(Resource)]
pub struct HexAssets {
    pub mesh: Handle<Mesh>,
}

pub fn run_if_damaged(damaged: Res<Damaged>) -> bool {
    damaged.0
}

pub fn damage(mut entered_event: EventReader<MapEvent>, mut damaged: ResMut<Damaged>) {
    for _event in entered_event.iter() {
        damaged.0 = true;
    }
}
