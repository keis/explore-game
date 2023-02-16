use super::{Damaged, Fog, GameMap, HexCoord, MapPosition, Zone};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct MapPresence {
    pub map: Entity,
    pub position: HexCoord,
}

#[derive(Component, Default)]
pub struct Offset(pub Vec3);

#[derive(Component, Default)]
pub struct ViewRadius(pub u32);

pub fn update_zone_visibility(
    view_query: Query<(&MapPresence, &ViewRadius)>,
    mut zone_query: Query<(&MapPosition, &mut Fog), With<Zone>>,
    mut damaged: ResMut<Damaged>,
) {
    for (position, mut fog) in zone_query.iter_mut() {
        fog.visible = view_query
            .iter()
            .any(|(presence, view_radius)| position.0.distance(presence.position) <= view_radius.0);
        if fog.visible {
            fog.explored = true;
        }
    }
    damaged.0 = false;
}

pub fn update_enemy_visibility(
    map_query: Query<&GameMap>,
    mut enemy_query: Query<&mut Visibility, Without<ViewRadius>>,
    zone_query: Query<(&MapPosition, &mut Fog), Changed<Fog>>,
) {
    let Ok(map) = map_query.get_single() else { return };
    for (position, fog) in &zone_query {
        let mut enemy_iter = enemy_query.iter_many_mut(map.presence(position.0));
        while let Some(mut visibility) = enemy_iter.fetch_next() {
            visibility.is_visible = fog.visible;
        }
    }
}
