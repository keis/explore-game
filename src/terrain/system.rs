use super::{asset::*, bundle::*, component::*, system_param::*};
use crate::{role::RoleCommandsExt, structure::Camp, ExplError};
use bevy::prelude::*;
use expl_codex::Id;
use expl_hexgrid::{Grid, Neighbours};
use expl_map::{Fog, MapEvent, MapLayout, MapPosition, MapPresence, PresenceLayer, ZoneLayer};

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
    let Ok(map) = map_query.single() else {
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
    let Ok((map, presence_layer)) = map_query.single() else {
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
        Query<(&MapPosition, &TerrainId), Without<Visibility>>,
        Query<(Entity, &TerrainId, &MapPosition, &Fog), Without<Visibility>>,
    )>,
    neighbour_fog_query: Query<&Fog>,
) -> Result<(), ExplError> {
    let (&MapLayout(layout), zone_layer) = map_query.single()?;

    let mut terrain_grid = Grid::<_, Id<Terrain>>::new(layout);
    for (position, terrain) in &zone_query.p0() {
        terrain_grid[position.0] = **terrain;
    }

    for (entity, terrain, position, fog) in &mut zone_query.p1() {
        let outer_visible = if fog.explored {
            OuterVisible::all_visible()
        } else {
            let mut bits = [false; 6];
            for (idx, coord) in position.0.neighbours().enumerate() {
                let Some(fog) = zone_layer
                    .get(coord)
                    .and_then(|&e| neighbour_fog_query.get(e).ok())
                else {
                    continue;
                };
                if fog.explored {
                    bits[idx % 6] = true;
                    bits[(idx + 5) % 6] = true;
                    bits[(idx + 1) % 6] = true;
                }
            }
            OuterVisible::with_data(bits)
        };

        let void = Id::from_tag("void");
        let neighbours = Neighbours::from_fn_around(position.0, |coord| {
            terrain_grid.get(coord).copied().unwrap_or(void)
        });

        commands.entity(entity).attach_role(ZoneRole::new(
            &mut zone_params,
            position,
            terrain,
            fog,
            outer_visible,
            neighbours,
        ));
    }
    Ok(())
}

pub fn decorate_zone(
    mut commands: Commands,
    zone_query: Query<(
        Entity,
        &TerrainId,
        &MapPosition,
        &Fog,
        &OuterTerrain,
        &ZoneDecorations,
    )>,
    mut zone_decoration_params: ZoneDecorationParams,
    mut water_params: WaterParams,
    terrain_codex: TerrainCodex,
    decoration_codex: DecorationCodex,
) -> Result<(), ExplError> {
    let terrain_codex = terrain_codex.get()?;
    let decoration_codex = decoration_codex.get()?;
    for (entity, terrain_id, position, fog, outer_terrain, zone_decorations) in &zone_query {
        let terrain = &terrain_codex[terrain_id];
        let height = Height::new(terrain_codex, **terrain_id, outer_terrain);
        commands.entity(entity).with_children(|parent| {
            for decoration in &terrain.decoration {
                match decoration {
                    TerrainDecoration::Crystal => {
                        if let Some(detail) = &zone_decorations.crystal_detail {
                            parent.spawn((
                                Name::new("Crystal"),
                                ZoneDecorationBundle::new(
                                    ZoneDecorationCrystals,
                                    Id::from_tag("crystal"),
                                    &mut zone_decoration_params,
                                    decoration_codex,
                                    &height,
                                    fog,
                                    **position,
                                    detail,
                                ),
                            ));
                        }
                    }
                    TerrainDecoration::Tree => {
                        for detail in &zone_decorations.tree_details {
                            parent.spawn((
                                Name::new("Tree"),
                                ZoneDecorationBundle::new(
                                    ZoneDecorationTree,
                                    Id::from_tag("tree"),
                                    &mut zone_decoration_params,
                                    decoration_codex,
                                    &height,
                                    fog,
                                    **position,
                                    detail,
                                ),
                            ));
                        }
                    }
                    TerrainDecoration::Water => {
                        parent.spawn((Name::new("Water"), WaterBundle::new(&mut water_params)));
                    }
                }
            }
        });
    }
    Ok(())
}

pub fn update_outer_visible(
    map_query: Query<&ZoneLayer>,
    changed_zone_query: Query<(Entity, &Fog, &MapPosition), Changed<Fog>>,
    mut zone_query: Query<(&Fog, &mut OuterVisible)>,
) {
    let Ok(map) = map_query.single() else {
        return;
    };
    for (entity, fog, position) in &changed_zone_query {
        if fog.explored {
            let Ok((_, mut outer_visible)) = zone_query.get_mut(entity) else {
                continue;
            };
            *outer_visible = OuterVisible::all_visible();

            for (idx, coord) in position.0.neighbours().enumerate() {
                let Some((neighbour_fog, mut neighbour_outer_visible)) =
                    map.get(coord).and_then(|&e| zone_query.get_mut(e).ok())
                else {
                    continue;
                };
                if neighbour_fog.explored {
                    continue;
                }
                neighbour_outer_visible[(idx + 2) % 6] = true;
                neighbour_outer_visible[(idx + 3) % 6] = true;
                neighbour_outer_visible[(idx + 4) % 6] = true;
            }
        }
    }
}
