use super::{bundle::*, component::*, system_param::*};
use crate::{
    actor::{ActorCodex, ActorParams, EnemyBundle, Members},
    creature::{CreatureCodex, Health},
    floating_text::{FloatingTextAlignment, FloatingTextPrototype, FloatingTextSource},
    material::PortalMaterial,
    role::RoleCommandsExt,
    scene::save,
    ExplError,
};
use bevy::{color::palettes::css, pbr::NotShadowCaster, prelude::*};
use expl_map::{Fog, MapCommandsExt, MapPresence, PresenceLayer, ViewRadius};

#[allow(clippy::type_complexity)]
pub fn fluff_structure(
    mut commands: Commands,
    mut structure_params: StructureParams,
    structure_codex: StructureCodex,
    structure_query: Query<(Entity, &StructureId, &MapPresence, &Fog), Without<Visibility>>,
) -> Result<(), ExplError> {
    let structure_codex = structure_codex.get()?;
    for (entity, structure_id, presence, fog) in &structure_query {
        commands.entity(entity).attach_role(StructureRole::new(
            &mut structure_params,
            structure_codex,
            **structure_id,
            presence,
            fog,
        ));
    }
    Ok(())
}

pub fn charge_spawner(mut spawner_query: Query<&mut Spawner>) {
    for mut spawner in &mut spawner_query {
        spawner.charge += 1;
    }
}

pub fn spawn_enemy(
    mut commands: Commands,
    mut spawner_query: Query<(&MapPresence, &mut Spawner)>,
    actor_codex: ActorCodex,
    creature_codex: CreatureCodex,
    presence_query: Query<Entity, Without<Spawner>>,
    map_query: Query<(Entity, &PresenceLayer)>,
    mut creature_params: ActorParams,
) -> Result<(), ExplError> {
    let actor_codex = actor_codex.get()?;
    let creature_codex = creature_codex.get()?;
    let (map_entity, presence_layer) = map_query.get_single()?;
    for (presence, mut spawner) in &mut spawner_query {
        if spawner.charge >= 3
            && presence_query
                .iter_many(presence_layer.presence(presence.position))
                .next()
                .is_none()
        {
            spawner.charge -= 3;
            info!("Spawning enemy at {} from {:?}", presence.position, spawner);
            let (enemy_bundle, actor_role) = EnemyBundle::new(
                presence.position,
                creature_codex,
                spawner.creature,
                spawner.actor,
            )
            .with_fluff(&mut creature_params, actor_codex);
            commands
                .entity(map_entity)
                .with_presence(presence.position, |location| {
                    location
                        .spawn((Name::new("Enemy"), save::Save, enemy_bundle))
                        .attach_role(actor_role);
                });
        }
    }

    Ok(())
}

pub fn update_portal_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut portal_materials: ResMut<Assets<PortalMaterial>>,
    portal_query: Query<(Entity, &Portal), Changed<Portal>>,
) {
    for (entity, portal) in &portal_query {
        if portal.open {
            commands
                .spawn((
                    NotShadowCaster,
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(2.0, 2.0))),
                    MeshMaterial3d(portal_materials.add(PortalMaterial {
                        base_color: Color::srgba(0.2, 0.7, 0.1, 0.3),
                        swirl_color: Color::srgba(0.4, 0.2, 0.7, 0.7),
                    })),
                    Transform::from_translation(Vec3::new(0.0, 0.9, 0.0)).with_rotation(
                        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                            * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                    ),
                ))
                .set_parent(entity);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_camp_view_radius(
    mut camp_query: Query<(&Members, &mut ViewRadius), (With<Camp>, Changed<Members>)>,
) {
    for (members, mut view_radius) in &mut camp_query {
        view_radius.0 = if members.is_empty() {
            0
        } else {
            ViewRadius::DEFAULT_VIEW_RADIUS
        };
    }
}

pub fn heal_characters(
    mut camp_query: Query<(&Members, &mut FloatingTextSource), With<Camp>>,
    mut health_query: Query<&mut Health>,
) {
    for (members, mut floating_text_source) in &mut camp_query {
        let mut iter = health_query.iter_many_mut(members.iter());
        while let Some(mut health) = iter.fetch_next() {
            let healed = health.heal(2);
            if healed > 0 {
                floating_text_source.add(FloatingTextPrototype {
                    value: healed.to_string(),
                    alignment: FloatingTextAlignment::Center,
                    color: css::GREEN.into(),
                });
            }
        }
    }
}
