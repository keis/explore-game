use super::{Damaged, Fog, GameMap, HexCoord, MapPosition, Zone};
use crate::enemy::Enemy;
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
    view_query: Query<(&MapPresence, &ViewRadius), Without<Enemy>>,
    mut zone_query: Query<(&MapPosition, &mut Fog), With<Zone>>,
    mut damaged: ResMut<Damaged>,
) {
    for (position, mut fog) in zone_query.iter_mut() {
        let visible = view_query
            .iter()
            .any(|(presence, view_radius)| position.0.distance(presence.position) <= view_radius.0);

        if visible != fog.visible {
            fog.visible = visible;
            if fog.visible {
                fog.explored = true;
            }
        }
    }
    damaged.0 = false;
}

#[allow(clippy::type_complexity)]
pub fn update_enemy_visibility(
    map_query: Query<&GameMap>,
    mut enemy_params: ParamSet<(
        Query<&mut Visibility, With<Enemy>>,
        Query<(&MapPresence, &mut Visibility), (With<Enemy>, Changed<MapPresence>)>,
    )>,
    changed_zone_query: Query<(&MapPosition, &Fog), Changed<Fog>>,
    any_zone_query: Query<&Fog>,
) {
    let Ok(map) = map_query.get_single() else { return };
    // Update enemies at locations that had their fog status changed
    for (position, fog) in &changed_zone_query {
        let mut enemy_query = enemy_params.p0();
        let mut enemy_iter = enemy_query.iter_many_mut(map.presence(position.0));
        while let Some(mut visibility) = enemy_iter.fetch_next() {
            visibility.is_visible = fog.visible;
        }
    }
    // Update enemies that had their location changed
    for (presence, mut visibility) in &mut enemy_params.p1() {
        let Some(fog) = map.get(presence.position).and_then(|&e| any_zone_query.get(e).ok()) else { continue };
        visibility.is_visible = fog.visible;
    }
}
