use crate::{
    actor::{EnemyBundle, EnemyParams},
    assets::MainAssets,
    map::{Fog, HexCoord, MapCommandsExt, MapPresence, PresenceLayer},
    material::TerrainMaterial,
    scene::save,
    terrain::HeightQuery,
};
use bevy::prelude::*;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Spawner {
    charge: u8,
}

pub type SpawnerParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<TerrainMaterial>>,
    HeightQuery<'w, 's>,
);

#[derive(Bundle, Default)]
pub struct SpawnerBundle {
    presence: MapPresence,
    fog: Fog,
    spawner: Spawner,
}

#[derive(Bundle, Default)]
pub struct SpawnerFluffBundle {
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl SpawnerBundle {
    pub fn new(position: HexCoord) -> Self {
        Self {
            presence: MapPresence { position },
            ..default()
        }
    }

    pub fn with_fluff(self, spawner_params: &mut SpawnerParams) -> (Self, SpawnerFluffBundle) {
        let fluff = SpawnerFluffBundle::new(spawner_params, &self.presence, &self.fog);
        (self, fluff)
    }
}

impl SpawnerFluffBundle {
    pub fn new(
        (main_assets, terrain_materials, height_query): &mut SpawnerParams,
        presence: &MapPresence,
        fog: &Fog,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.portal_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.8, 0.32, 0.3),
                    ..default()
                }),
                visibility: if fog.visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                transform: Transform::from_translation(
                    height_query.adjust(presence.position.into()),
                )
                .with_scale(Vec3::splat(0.3))
                .with_rotation(Quat::from_rotation_y(2.0)),
                ..default()
            },
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn fluff_spawner(
    mut commands: Commands,
    mut spawner_params: SpawnerParams,
    spawner_query: Query<(Entity, &MapPresence, &Fog), (With<Spawner>, Without<GlobalTransform>)>,
) {
    for (entity, presence, fog) in &spawner_query {
        commands
            .entity(entity)
            .insert(SpawnerFluffBundle::new(&mut spawner_params, presence, fog));
    }
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
    let Ok((map_entity, presence_layer)) = map_query.get_single() else { return };
    for (presence, mut spawner) in &mut spawner_query {
        if spawner.charge >= 3
            && presence_query
                .iter_many(presence_layer.presence(presence.position))
                .next()
                .is_none()
        {
            spawner.charge -= 3;
            info!("Spawning enemy at {}", presence.position);
            commands
                .entity(map_entity)
                .with_presence(presence.position, |location| {
                    location.spawn((
                        Name::new("Enemy"),
                        save::Save,
                        EnemyBundle::new(presence.position).with_fluff(&mut enemy_params),
                    ));
                });
        }
    }
}
