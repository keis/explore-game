use super::{Damaged, HexCoord, MapPosition};
use crate::fog::Fog;
use crate::zone::Zone;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct MapPresence {
    pub map: Entity,
    pub position: HexCoord,
}

#[derive(Component)]
pub struct Offset(pub Vec3);

#[derive(Component)]
pub struct ViewRadius(pub usize);

pub fn update_visibility(
    presence_query: Query<(&MapPresence, &ViewRadius)>,
    mut zone_query: Query<(&MapPosition, &mut Fog), With<Zone>>,
    mut damaged: ResMut<Damaged>,
) {
    for (position, mut fog) in zone_query.iter_mut() {
        fog.visible = presence_query.iter().any(|(presence, view_radius)| {
            position.0.distance(&presence.position) <= view_radius.0
        });
        fog.explored = fog.explored || fog.visible;
    }
    damaged.0 = false;
}
