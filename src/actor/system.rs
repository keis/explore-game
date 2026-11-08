use super::{bundle::*, component::*, event::*};
use crate::{
    role::{AssumedRoles, RoleCommandsExt},
    terrain::HeightQuery,
    ExplError,
};
use bevy::prelude::*;
use expl_map::{Fog, MapCommandsExt, MapPosition, MapPresence, PresenceLayer, ZoneLayer};

pub fn fluff_actor(
    mut commands: Commands,
    actor_query: Query<(Entity, &ActorId), Without<AssumedRoles>>,
) {
    for (entity, actor_id) in &actor_query {
        commands
            .entity(entity)
            .attach_role(ActorRole::new(**actor_id));
    }
}

#[allow(clippy::type_complexity)]
pub fn fluff_party(
    mut commands: Commands,
    party_query: Query<Entity, (With<Party>, Without<AssumedRoles>)>,
) -> Result<(), ExplError> {
    for entity in &party_query {
        commands.entity(entity).attach_role(PartyRole::default());
    }
    Ok(())
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
    let Ok((zone_layer, presence_layer)) = map_query.single() else {
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
    trigger: On<Remove, Members>,
    party_query: Query<(&Party, &MapPresence)>,
    map_query: Query<Entity, With<PresenceLayer>>,
    mut commands: Commands,
) -> Result<(), ExplError> {
    if party_query.get(trigger.entity).is_err() {
        return Ok(());
    }
    let map_entity = map_query.single()?;
    commands.entity(map_entity).despawn_presence(trigger.entity);
    Ok(())
}

const SLIDE_SPEED: f32 = 1.7;

pub fn slide(
    mut slide_query: Query<(&mut Transform, &mut Slide)>,
    height_query: HeightQuery,
    mut events: MessageWriter<SlideEvent>,
    time: Res<Time>,
) {
    for (mut transform, mut slide) in slide_query.iter_mut() {
        if slide.progress == 1.0 {
            continue;
        }
        slide.progress = (slide.progress + time.delta_secs() * SLIDE_SPEED).clamp(0.0, 1.0);
        let position = slide.start.lerp(
            slide.end,
            EaseFunction::QuadraticInOut.sample_clamped(slide.progress),
        );
        transform.translation = height_query.adjust(position);
        if slide.progress == 1.0 {
            events.write(SlideEvent::Stopped);
        }
    }
}
