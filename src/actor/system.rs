use super::{bundle::*, component::*, event::*, system_param::*};
use crate::{role::RoleCommandsExt, terrain::HeightQuery, ExplError};
use bevy::prelude::*;
use expl_map::{Fog, MapCommandsExt, MapPosition, MapPresence, PresenceLayer, ZoneLayer};
use interpolation::Ease;

#[allow(clippy::type_complexity)]
pub fn fluff_actor(
    mut commands: Commands,
    actor_codex: ActorCodex,
    creature_query: Query<(Entity, &ActorId, &MapPresence), Without<Visibility>>,
    mut creature_params: ActorParams,
) -> Result<(), ExplError> {
    let actor_codex = actor_codex.get()?;
    for (entity, creature_id, presence) in &creature_query {
        commands.entity(entity).attach_role(ActorRole::new(
            &mut creature_params,
            actor_codex,
            **creature_id,
            presence,
        ));
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn fluff_party(
    mut commands: Commands,
    party_query: Query<Entity, (With<Party>, Without<Visibility>)>,
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
    trigger: Trigger<MemberRemoved>,
    party_query: Query<&Members, (With<Party>, With<MapPresence>)>,
    map_query: Query<Entity, With<PresenceLayer>>,
    mut commands: Commands,
) -> Result<(), ExplError> {
    let map_entity = map_query.single()?;
    let members = party_query.get(trigger.target())?;
    if members.is_empty() {
        commands
            .entity(map_entity)
            .despawn_presence(trigger.target());
    }
    Ok(())
}

const SLIDE_SPEED: f32 = 1.7;

pub fn slide(
    mut slide_query: Query<(&mut Transform, &mut Slide)>,
    height_query: HeightQuery,
    mut events: EventWriter<SlideEvent>,
    time: Res<Time>,
) {
    for (mut transform, mut slide) in slide_query.iter_mut() {
        if slide.progress == 1.0 {
            continue;
        }
        slide.progress = (slide.progress + time.delta_secs() * SLIDE_SPEED).clamp(0.0, 1.0);
        let position = slide
            .start
            .lerp(slide.end, slide.progress.quadratic_in_out());
        transform.translation = height_query.adjust(position);
        if slide.progress == 1.0 {
            events.write(SlideEvent::Stopped);
        }
    }
}
