use super::{Damaged, HexCoord};
use crate::Fog;
use crate::Zone;
use bevy::prelude::*;

#[derive(Component)]
pub struct MapPresence {
    pub map: Entity,
    pub position: HexCoord,
    pub offset: Vec3,
    pub view_radius: usize,
}

pub fn update_visibility(
    presence_query: Query<&MapPresence>,
    mut zone_query: Query<(&Zone, &mut Fog)>,
    mut damaged: ResMut<Damaged>,
) {
    for (zone, mut fog) in zone_query.iter_mut() {
        fog.visible = presence_query
            .iter()
            .any(|presence| zone.position.distance(&presence.position) <= presence.view_radius);
        fog.explored = fog.explored || fog.visible;
    }
    damaged.0 = false;
}
