use super::{bundle::*, component::*};
use crate::{
    map::{Fog, MapEvent, MapLayout, MapPosition, MapPresence, PresenceLayer, ZoneLayer},
    structure::Camp,
};
use bevy::prelude::*;
use expl_hexgrid::Grid;

pub fn despawn_empty_crystal_deposit(
    mut commands: Commands,
    crystal_deposit_query: Query<(&CrystalDeposit, &Children), Changed<CrystalDeposit>>,
    zone_decoration_query: Query<Entity, With<ZoneDecorationCrystals>>,
) {
    for (_, children) in crystal_deposit_query
        .iter()
        .filter(|(deposit, _)| deposit.amount == 0)
    {
        for decoration_entity in zone_decoration_query.iter_many(children.iter()) {
            commands.entity(decoration_entity).despawn();
        }
    }
}

pub fn hide_decorations_behind_camp(
    presence_query: Query<&MapPresence, (Changed<MapPresence>, With<Camp>)>,
    map_query: Query<&ZoneLayer>,
    zone_query: Query<&Children>,
    mut decoration_query: Query<(&mut Visibility, &Transform), With<ZoneDecorationTree>>,
) {
    let Ok(map) = map_query.get_single() else {
        return;
    };
    for presence in &presence_query {
        let Some(children) = map
            .get(presence.position)
            .and_then(|&e| zone_query.get(e).ok())
        else {
            continue;
        };
        let mut decoration_iter = decoration_query.iter_many_mut(children);
        while let Some((mut visibility, transform)) = decoration_iter.fetch_next() {
            if transform.translation.distance(Vec3::ZERO) < 0.3 {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn show_decorations_behind_camp(
    mut events: EventReader<MapEvent>,
    map_query: Query<(&ZoneLayer, &PresenceLayer)>,
    zone_query: Query<&Children>,
    camp_query: Query<&Camp>,
    mut decoration_query: Query<&mut Visibility, With<ZoneDecorationTree>>,
) {
    let Ok((map, presence_layer)) = map_query.get_single() else {
        return;
    };
    for event in events.read() {
        let MapEvent::PresenceRemoved { position, .. } = event else {
            continue;
        };
        if camp_query
            .iter_many(presence_layer.presence(*position))
            .next()
            .is_some()
        {
            continue;
        }
        let Some(children) = map.get(*position).and_then(|&e| zone_query.get(e).ok()) else {
            continue;
        };
        let mut decoration_iter = decoration_query.iter_many_mut(children);
        while let Some(mut visibility) = decoration_iter.fetch_next() {
            *visibility = Visibility::Inherited;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn fluff_zone(
    mut commands: Commands,
    mut zone_params: ZoneParams,
    map_query: Query<(&MapLayout, &ZoneLayer)>,
    mut zone_query: ParamSet<(
        Query<(&MapPosition, &Height), Without<GlobalTransform>>,
        Query<(Entity, &Terrain, &MapPosition, &mut Height, &Fog), Without<GlobalTransform>>,
    )>,
    neighbour_zone_query: Query<&Fog>,
) {
    let Ok((&MapLayout(layout), zone_layer)) = map_query.get_single() else {
        return;
    };

    let mut height_grid = Grid::<_, (f32, f32)>::new(layout);
    for (position, height) in &zone_query.p0() {
        height_grid[position.0] = (height.height_amp, height.height_base);
    }

    for (entity, terrain, position, mut height, fog) in &mut zone_query.p1() {
        for (idx, coord) in position.0.neighbours().enumerate() {
            let (amp, base) = height_grid.get(coord).unwrap_or(&(0.0, 0.0));
            height.outer_amp[idx] = *amp;
            height.outer_base[idx] = *base;
        }

        let outer_visible = if fog.explored {
            OuterVisible([true; 6])
        } else {
            let mut bits = [false; 6];
            for (idx, coord) in position.0.neighbours().enumerate() {
                let Some(fog) = zone_layer
                    .get(coord)
                    .and_then(|&e| neighbour_zone_query.get(e).ok())
                else {
                    continue;
                };
                if fog.explored {
                    bits[idx % 6] = true;
                    bits[(idx + 5) % 6] = true;
                    bits[(idx + 1) % 6] = true;
                }
            }
            OuterVisible(bits)
        };
        commands.entity(entity).insert(ZoneFluffBundle::new(
            &mut zone_params,
            position,
            terrain,
            &height,
            fog,
            outer_visible,
        ));
    }
}

pub fn decorate_zone(
    mut commands: Commands,
    zone_query: Query<(
        Entity,
        &Terrain,
        &MapPosition,
        &Height,
        &Fog,
        &ZoneDecorations,
    )>,
    mut zone_decoration_params: ZoneDecorationParams,
    mut water_params: WaterParams,
) {
    for (entity, terrain, position, height, fog, zone_decorations) in &zone_query {
        commands.entity(entity).with_children(|parent| {
            if let Some(ZoneDecorationDetail(pos, scale)) = zone_decorations.crystal_detail {
                parent.spawn((
                    Name::new("Crystal"),
                    ZoneDecorationCrystalsBundle::new(
                        &mut zone_decoration_params,
                        height,
                        fog,
                        **position,
                        pos,
                        scale,
                    ),
                ));
            }
            for ZoneDecorationDetail(pos, scale) in &zone_decorations.tree_details {
                parent.spawn((
                    Name::new("Tree"),
                    ZoneDecorationTreeBundle::new(
                        &mut zone_decoration_params,
                        height,
                        fog,
                        **position,
                        *pos,
                        *scale,
                    ),
                ));
            }
            if *terrain == Terrain::Ocean {
                parent.spawn((Name::new("Water"), WaterBundle::new(&mut water_params)));
            }
        });
    }
}

pub fn update_outer_visible(
    map_query: Query<&ZoneLayer>,
    changed_zone_query: Query<(Entity, &Fog, &MapPosition), Changed<Fog>>,
    mut zone_query: Query<(&Fog, &mut OuterVisible)>,
) {
    let Ok(map) = map_query.get_single() else {
        return;
    };
    for (entity, fog, position) in &changed_zone_query {
        if fog.explored {
            let Ok((_, mut outer_visible)) = zone_query.get_mut(entity) else {
                continue;
            };
            outer_visible.0 = [true; 6];

            for (idx, coord) in position.0.neighbours().enumerate() {
                let Some((neighbour_fog, mut neighbour_outer_visible)) =
                    map.get(coord).and_then(|&e| zone_query.get_mut(e).ok())
                else {
                    continue;
                };
                if neighbour_fog.explored {
                    continue;
                }
                neighbour_outer_visible.0[(idx + 2) % 6] = true;
                neighbour_outer_visible.0[(idx + 3) % 6] = true;
                neighbour_outer_visible.0[(idx + 4) % 6] = true;
            }
        }
    }
}
