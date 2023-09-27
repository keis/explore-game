use super::{bundle::*, component::*};
use crate::{
    crystals::CrystalDeposit,
    map::{Fog, MapEvent, MapPosition, MapPresence, PresenceLayer, ZoneLayer},
    structure::Camp,
};
use bevy::prelude::*;

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
    let Ok(map) = map_query.get_single() else { return };
    for presence in &presence_query {
        let Some(children) = map.get(presence.position).and_then(|&e| zone_query.get(e).ok()) else { continue };
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
    let Ok((map, presence_layer)) = map_query.get_single() else { return };
    for event in &mut events {
        let MapEvent::PresenceRemoved { position, .. } = event else { continue };
        if camp_query
            .iter_many(presence_layer.presence(*position))
            .next()
            .is_some()
        {
            continue;
        }
        let Some(children) = map.get(*position).and_then(|&e| zone_query.get(e).ok()) else { continue };
        let mut decoration_iter = decoration_query.iter_many_mut(children);
        while let Some(mut visibility) = decoration_iter.fetch_next() {
            *visibility = Visibility::Inherited;
        }
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
