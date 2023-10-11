use super::{bundle::*, component::*, event::*, system_param::*};
use crate::{
    action::{GameAction, GameActionQueue},
    map::{
        Fog, HexCoord, MapCommandsExt, MapPosition, MapPresence, Offset, PresenceLayer, ViewRadius,
        ZoneLayer,
    },
    path::PathFinder,
    terrain::{HeightQuery, Terrain},
};
use bevy::prelude::*;
use interpolation::Ease;
use rand::{seq::SliceRandom, thread_rng};

pub fn reset_movement_points(mut movement_query: Query<&mut Movement>) {
    for mut movement in movement_query.iter_mut() {
        movement.points = 2;
    }
}

#[allow(clippy::type_complexity)]
pub fn fluff_enemy(
    mut commands: Commands,
    enemy_query: Query<(Entity, &MapPresence, &Offset), (With<Enemy>, Without<GlobalTransform>)>,
    mut enemy_params: EnemyParams,
) {
    for (entity, presence, offset) in &enemy_query {
        commands
            .entity(entity)
            .insert(EnemyFluffBundle::new(&mut enemy_params, presence, offset));
    }
}

#[allow(clippy::type_complexity)]
pub fn fluff_party(
    mut commands: Commands,
    party_query: Query<(Entity, &MapPresence, &Offset), (With<Party>, Without<GlobalTransform>)>,
    mut party_params: PartyParams,
) {
    for (entity, presence, offset) in &party_query {
        commands
            .entity(entity)
            .insert(PartyFluffBundle::new(&mut party_params, presence, offset));
    }
}

pub fn move_enemy(
    mut queue: ResMut<GameActionQueue>,
    map_query: Query<(&PresenceLayer, &ZoneLayer)>,
    enemy_query: Query<(Entity, &MapPresence, &ViewRadius), With<Enemy>>,
    terrain_query: Query<&Terrain>,
    target: Target,
    path_finder: PathFinder,
) {
    let Ok((presence_layer, zone_layer)) = map_query.get_single() else {
        return;
    };
    let mut rng = thread_rng();
    for (entity, presence, view_radius) in &enemy_query {
        if let Some(target) = target.closest_in_view(presence.position, view_radius) {
            let Some((path, _length)) = path_finder.find_path(presence.position, target.position)
            else {
                continue;
            };
            let Some(next) = path.get(1) else { continue };
            if enemy_query
                .iter_many(presence_layer.presence(*next))
                .next()
                .is_some()
            {
                continue;
            }
            queue.add(GameAction::Move(entity, *next));
        } else {
            let mut neighbours = HexCoord::NEIGHBOUR_OFFSETS;
            neighbours.shuffle(&mut rng);
            for offset in neighbours {
                let next = presence.position + offset;
                if zone_layer
                    .get(next)
                    .and_then(|&entity| terrain_query.get(entity).ok())
                    .map_or(false, |terrain| terrain.is_walkable())
                    && enemy_query
                        .iter_many(presence_layer.presence(next))
                        .next()
                        .is_none()
                {
                    queue.add(GameAction::Move(entity, next));
                    break;
                }
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
    let Ok((zone_layer, presence_layer)) = map_query.get_single() else {
        return;
    };
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
        let Some(fog) = zone_layer
            .get(presence.position)
            .and_then(|&e| any_zone_query.get(e).ok())
        else {
            continue;
        };
        *visibility = if fog.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

#[allow(clippy::type_complexity)]
pub fn despawn_empty_party(
    mut commands: Commands,
    party_query: Query<(Entity, &Group), (With<Party>, With<MapPresence>, Changed<Group>)>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(map_entity) = map_query.get_single() else {
        return;
    };
    for (entity, group) in &party_query {
        if group.members.is_empty() {
            commands.entity(map_entity).despawn_presence(entity);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn derive_party_movement(
    mut party_query: Query<(&Group, &mut Movement), (With<Party>, Changed<Group>)>,
    movement_query: Query<&Movement, Without<Party>>,
) {
    for (group, mut party_movement) in party_query.iter_mut() {
        party_movement.points = movement_query
            .iter_many(&group.members)
            .map(|m| m.points)
            .min()
            .unwrap_or(0);
    }
}

const SLIDE_SPEED: f32 = 1.7;

pub fn slide(
    mut slide_query: Query<(&mut Transform, &mut Slide, &Offset)>,
    height_query: HeightQuery,
    mut events: EventWriter<SlideEvent>,
    time: Res<Time>,
) {
    for (mut transform, mut slide, offset) in slide_query.iter_mut() {
        if slide.progress == 1.0 {
            continue;
        }
        slide.progress = (slide.progress + time.delta_seconds() * SLIDE_SPEED).clamp(0.0, 1.0);
        let position = slide
            .start
            .lerp(slide.end, slide.progress.quadratic_in_out());
        transform.translation = height_query.adjust(position) + offset.0;
        if slide.progress == 1.0 {
            events.send(SlideEvent::Stopped);
        }
    }
}
