use super::{events::Entered, HexCoord, MapComponent, MapPresence, Offset};
use crate::hex::coord_to_vec3;
use crate::party::Party;
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component)]
pub struct PathGuided {
    progress: f32,
    path: VecDeque<HexCoord>,
}

impl PathGuided {
    pub fn path(&mut self, path: Vec<HexCoord>) {
        self.path = VecDeque::from(path);
        self.path.pop_front();
    }
}

impl Default for PathGuided {
    fn default() -> Self {
        PathGuided {
            progress: 0.0,
            path: VecDeque::new(),
        }
    }
}

pub fn progress_path_guided(
    time: Res<Time>,
    mut map_query: Query<&mut MapComponent>,
    mut positioned_query: Query<(
        Entity,
        &mut PathGuided,
        &mut Party,
        &mut MapPresence,
        &mut Transform,
        &Offset,
    )>,
    mut hex_entered_event: EventWriter<Entered>,
) {
    // TODO: Decouple this from party
    for (entity, mut pathguided, mut party, mut positioned, mut transform, offset) in
        positioned_query.iter_mut()
    {
        if pathguided.path.is_empty() || party.movement_points == 0 {
            continue;
        }

        let mut map = map_query
            .get_mut(positioned.map)
            .expect("references valid map");
        pathguided.progress += time.delta_seconds();
        if pathguided.progress >= 1.0 {
            let newposition = pathguided.path.pop_front().expect("path has element");
            map.storage
                .move_presence(entity, positioned.position, newposition);
            positioned.position = newposition;
            hex_entered_event.send(Entered {
                entity,
                coordinate: positioned.position,
            });
            party.movement_points -= 1;
            pathguided.progress = 0.0;
        }

        if let Some(next) = pathguided.path.front() {
            let orig_translation = coord_to_vec3(positioned.position, map.radius) + offset.0;
            let new_translation = coord_to_vec3(*next, map.radius) + offset.0;
            transform.translation = orig_translation.lerp(new_translation, pathguided.progress);
        }
    }
}
