use super::{HexCoord, MapComponent, MapPresence, MoveMapPresence, Offset};
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
    mut commands: Commands,
    map_query: Query<(Entity, &MapComponent)>,
    mut path_guided_query: Query<(
        Entity,
        &mut PathGuided,
        &mut Party,
        &MapPresence,
        &mut Transform,
        &Offset,
    )>,
) {
    // TODO: Decouple this from party
    for (entity, mut pathguided, mut party, presence, mut transform, offset) in
        path_guided_query.iter_mut()
    {
        if pathguided.path.is_empty() || party.movement_points == 0 {
            continue;
        }

        let (map_entity, map) = map_query.get(presence.map).expect("references valid map");
        pathguided.progress += time.delta_seconds();
        if pathguided.progress >= 1.0 {
            let newposition = pathguided.path.pop_front().expect("path has element");
            commands.add(MoveMapPresence {
                map: map_entity,
                presence: entity,
                position: newposition,
            });
            party.movement_points -= 1;
            pathguided.progress = 0.0;
        }

        if let Some(next) = pathguided.path.front() {
            let orig_translation = coord_to_vec3(presence.position, map.radius) + offset.0;
            let new_translation = coord_to_vec3(*next, map.radius) + offset.0;
            transform.translation = orig_translation.lerp(new_translation, pathguided.progress);
        }
    }
}
