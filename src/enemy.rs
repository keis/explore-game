use crate::{
    assets::MainAssets,
    map::{coord_to_vec3, HexCoord, Offset},
};
use bevy::prelude::*;

pub struct EnemyBundle {
    pub offset: Offset,
}

pub fn spawn_enemy(
    commands: &mut Commands,
    params: &mut ParamSet<(Res<MainAssets>, ResMut<Assets<StandardMaterial>>)>,
    position: HexCoord,
) -> Entity {
    let offset = Vec3::ZERO;
    commands
        .spawn((PbrBundle {
            mesh: params.p0().blob_mesh.clone(),
            material: params
                .p1()
                .add(Color::rgba(0.749, 0.584, 0.901, 0.666).into()),
            transform: Transform::from_translation(coord_to_vec3(position) + offset),
            ..default()
        },))
        .id()
}
