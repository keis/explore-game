use super::{Damaged, Fog, HexCoord, MapCommandsExt, MapPosition, ZoneLayer};
use crate::{actor::enemy::Enemy, terrain::Terrain};
use bevy::prelude::*;
use expl_hexgrid::{layout::SquareGridLayout, Grid};
use std::collections::hash_set::HashSet;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct MapPresence {
    pub position: HexCoord,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Offset(pub Vec3);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ViewRadius(pub u32);

#[derive(Component)]
pub struct PresenceLayer {
    presence: Grid<SquareGridLayout, HashSet<Entity>>,
    void: HashSet<Entity>,
}

impl PresenceLayer {
    pub fn new(layout: SquareGridLayout) -> Self {
        PresenceLayer {
            presence: Grid::new(layout),
            void: HashSet::new(),
        }
    }

    pub fn presence(&self, position: HexCoord) -> impl Iterator<Item = &Entity> {
        self.presence
            .get(position)
            .map_or_else(|| self.void.iter(), |presence| presence.iter())
    }

    pub fn add_presence(&mut self, position: HexCoord, entity: Entity) {
        if let Some(presence) = self.presence.get_mut(position) {
            presence.insert(entity);
        }
    }

    pub fn remove_presence(&mut self, position: HexCoord, entity: Entity) {
        if let Some(presence) = self.presence.get_mut(position) {
            presence.remove(&entity);
        }
    }

    pub fn move_presence(&mut self, entity: Entity, origin: HexCoord, destination: HexCoord) {
        if let Some(o) = self.presence.get_mut(origin) {
            o.remove(&entity);
        }
        if let Some(d) = self.presence.get_mut(destination) {
            d.insert(entity);
        }
    }
}

pub fn update_zone_visibility(
    view_query: Query<(&MapPresence, &ViewRadius), Without<Enemy>>,
    mut zone_query: Query<(&MapPosition, &mut Fog), With<Terrain>>,
    mut damaged: ResMut<Damaged>,
) {
    for (position, mut fog) in zone_query.iter_mut() {
        let visible = view_query
            .iter()
            .any(|(presence, view_radius)| position.0.distance(presence.position) < view_radius.0);

        if visible != fog.visible {
            fog.visible = visible;
            if fog.visible {
                fog.explored = true;
            }
        }
    }
    damaged.0 = false;
}

pub fn update_terrain_visibility(
    zone_query: Query<(&Children, &Fog), Changed<Fog>>,
    mut terrain_query: Query<(&mut Fog, &mut Visibility), Without<Children>>,
) {
    for (children, parent_fog) in &zone_query {
        let mut terrain_iter = terrain_query.iter_many_mut(children);
        while let Some((mut fog, mut visibility)) = terrain_iter.fetch_next() {
            fog.visible = parent_fog.visible;
            fog.explored = parent_fog.explored;

            *visibility = if fog.explored {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_presence_fog(
    zone_query: Query<(&MapPosition, &Fog), (Changed<Fog>, Without<MapPresence>)>,
    map_query: Query<&PresenceLayer>,
    mut presence_query: Query<(&mut Fog, &mut Visibility), With<MapPresence>>,
) {
    let Ok(presence_layer) = map_query.get_single() else { return };
    for (position, zone_fog) in &zone_query {
        let mut presence_iter = presence_query.iter_many_mut(presence_layer.presence(position.0));
        while let Some((mut fog, mut visibility)) = presence_iter.fetch_next() {
            fog.visible = zone_fog.visible;
            fog.explored = zone_fog.explored;

            if fog.explored {
                *visibility = Visibility::Inherited;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_enemy_visibility(
    map_query: Query<(&ZoneLayer, &PresenceLayer)>,
    mut enemy_params: ParamSet<(
        Query<&mut Visibility, With<Enemy>>,
        Query<(&MapPresence, &mut Visibility), (With<Enemy>, Changed<MapPresence>)>,
    )>,
    changed_zone_query: Query<(&MapPosition, &Fog), Changed<Fog>>,
    any_zone_query: Query<&Fog>,
) {
    let Ok((zone_layer, presence_layer)) = map_query.get_single() else { return };
    // Update enemies at locations that had their fog status changed
    for (position, fog) in &changed_zone_query {
        let mut enemy_query = enemy_params.p0();
        let mut enemy_iter = enemy_query.iter_many_mut(presence_layer.presence(position.0));
        while let Some(mut visibility) = enemy_iter.fetch_next() {
            *visibility = if fog.visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
    // Update enemies that had their location changed
    for (presence, mut visibility) in &mut enemy_params.p1() {
        let Some(fog) = zone_layer.get(presence.position).and_then(|&e| any_zone_query.get(e).ok()) else { continue };
        *visibility = if fog.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn fluff_presence(
    mut commands: Commands,
    map_query: Query<Entity, With<PresenceLayer>>,
    presence_query: Query<(Entity, &MapPresence), Without<GlobalTransform>>,
) {
    let Ok(map_entity) = map_query.get_single() else { return };
    for (entity, presence) in &presence_query {
        commands
            .entity(map_entity)
            .add_presence(entity, presence.position);
    }
}
