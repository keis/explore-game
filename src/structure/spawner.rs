use crate::{
    assets::MainAssets,
    enemy::{EnemyBundle, EnemyParams},
    map::{Fog, HeightQuery, HexCoord, MapCommandsExt, MapPresence, PresenceLayer},
    material::TerrainMaterial,
};
use bevy::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Spawner {
    charge: u8,
}

#[derive(Bundle, Default)]
pub struct SpawnerBundle {
    fog: Fog,
    spawner: Spawner,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

pub type SpawnerParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<TerrainMaterial>>,
    HeightQuery<'w, 's>,
);

impl SpawnerBundle {
    pub fn new(
        (main_assets, terrain_materials, height_query): &mut SpawnerParams,
        position: HexCoord,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.portal_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.8, 0.32, 0.3),
                    ..default()
                }),
                visibility: Visibility::Hidden,
                transform: Transform::from_translation(height_query.adjust(position.into()))
                    .with_scale(Vec3::splat(0.3))
                    .with_rotation(Quat::from_rotation_y(2.0)),
                ..default()
            },
            ..default()
        }
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
                    location.spawn(EnemyBundle::new(&mut enemy_params, presence.position));
                });
        }
    }
}
