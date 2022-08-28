use super::{events::Entered, HexCoord, MapComponent, MapPresence};
use crate::hex::coord_to_vec3;
use crate::Turn;
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component)]
pub struct PathGuided {
    progress: f32,
    path: VecDeque<HexCoord>,
    movement_points: u32,
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
            movement_points: 2,
            path: VecDeque::new(),
        }
    }
}

pub fn reset_movement_points(turn: Res<Turn>, mut path_guided_query: Query<&mut PathGuided>) {
    if turn.is_changed() {
        for mut path_guided in path_guided_query.iter_mut() {
            path_guided.movement_points = 2;
        }
    }
}

pub fn progress_path_guided(
    time: Res<Time>,
    mut map_query: Query<&mut MapComponent>,
    mut positioned_query: Query<(Entity, &mut PathGuided, &mut MapPresence, &mut Transform)>,
    mut hex_entered_event: EventWriter<Entered>,
) {
    for (entity, mut pathguided, mut positioned, mut transform) in positioned_query.iter_mut() {
        if pathguided.path.is_empty() || pathguided.movement_points == 0 {
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
            pathguided.movement_points -= 1;
            pathguided.progress = 0.0;
        }

        if let Some(next) = pathguided.path.front() {
            let orig_translation =
                coord_to_vec3(positioned.position, map.radius) + positioned.offset;
            let new_translation = coord_to_vec3(*next, map.radius) + positioned.offset;
            transform.translation = orig_translation.lerp(new_translation, pathguided.progress);
        }
    }
}
