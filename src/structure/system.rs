use super::{bundle::*, component::*, system_param::*};
use crate::{
    actor::{EnemyBundle, EnemyParams, Group},
    combat::Health,
    floating_text::{FloatingTextAlignment, FloatingTextPrototype, FloatingTextSource},
    map::{Fog, MapCommandsExt, MapPresence, PresenceLayer, ViewRadius},
    material::PortalMaterial,
    scene::save,
    ExplError,
};
use bevy::{pbr::NotShadowCaster, prelude::*};

#[allow(clippy::type_complexity)]
pub fn fluff_structure(
    mut commands: Commands,
    mut structure_params: StructureParams,
    structure_codex: StructureCodex,
    structure_query: Query<(Entity, &StructureId, &MapPresence, &Fog), Without<GlobalTransform>>,
) -> Result<(), ExplError> {
    let structure_codex = structure_codex.get()?;
    for (entity, structure_id, presence, fog) in &structure_query {
        commands.entity(entity).insert(StructureFluffBundle::new(
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
    presence_query: Query<Entity, Without<Spawner>>,
    map_query: Query<(Entity, &PresenceLayer)>,
    mut enemy_params: EnemyParams,
) {
    let Ok((map_entity, presence_layer)) = map_query.get_single() else {
        return;
    };
    for (presence, mut spawner) in &mut spawner_query {
        if spawner.charge >= 3
            && presence_query
                .iter_many(presence_layer.presence(presence.position))
                .next()
                .is_none()
        {
            spawner.charge -= 3;
            info!("Spawning enemy at {}", presence.position);
            let (fluff_bundle, child_bundle) =
                EnemyBundle::new(presence.position).with_fluff(&mut enemy_params);
            commands
                .entity(map_entity)
                .with_presence(presence.position, |location| {
                    location
                        .spawn((Name::new("Enemy"), save::Save, fluff_bundle))
                        .with_children(|parent| {
                            parent.spawn(child_bundle);
                        });
                });
        }
    }
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
                    MaterialMeshBundle {
                        mesh: meshes.add(shape::Plane::from_size(2.0).into()),
                        material: portal_materials.add(PortalMaterial {
                            base_color: Color::rgba(0.2, 0.7, 0.1, 0.3),
                            swirl_color: Color::rgba(0.4, 0.2, 0.7, 0.7),
                        }),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.9, 0.0))
                            .with_rotation(
                                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                                    * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                            ),
                        ..default()
                    },
                ))
                .set_parent(entity);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_camp_view_radius(
    mut camp_query: Query<(&Group, &mut ViewRadius), (With<Camp>, Changed<Group>)>,
) {
    for (group, mut view_radius) in &mut camp_query {
        view_radius.0 = if group.members.is_empty() {
            0
        } else {
            ViewRadius::DEFAULT_VIEW_RADIUS
        };
    }
}

pub fn heal_characters(
    mut camp_query: Query<(&Group, &mut FloatingTextSource), With<Camp>>,
    mut health_query: Query<&mut Health>,
) {
    for (group, mut floating_text_source) in &mut camp_query {
        let mut iter = health_query.iter_many_mut(&group.members);
        while let Some(mut health) = iter.fetch_next() {
            let healed = health.heal(2);
            if healed > 0 {
                floating_text_source.add(FloatingTextPrototype {
                    value: healed.to_string(),
                    alignment: FloatingTextAlignment::Center,
                    color: Color::GREEN,
                });
            }
        }
    }
}
