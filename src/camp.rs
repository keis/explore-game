use crate::{
    assets::MainAssets,
    map::{coord_to_vec3, Fog, HexCoord, Offset, ViewRadius},
    material::TerrainMaterial,
    party::Group,
    VIEW_RADIUS,
};
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;

#[derive(Component, Debug, Default)]
pub struct Camp {
    pub name: String,
    pub supplies: u32,
    pub crystals: u32,
}

#[derive(Bundle, Default)]
pub struct CampBundle {
    pub camp: Camp,
    pub group: Group,
    pub pickable_bundle: PickableBundle,
    pub offset: Offset,
    pub view_radius: ViewRadius,
    pub fog: Fog,
}

pub fn spawn_camp(
    commands: &mut Commands,
    params: &mut ParamSet<(Res<MainAssets>, ResMut<Assets<TerrainMaterial>>)>,
    position: HexCoord,
    camp: Camp,
) -> Entity {
    commands
        .spawn((
            MaterialMeshBundle {
                mesh: params.p0().tent_mesh.clone(),
                material: params.p1().add(TerrainMaterial {
                    color: Color::rgb(0.631, 0.596, 0.165),
                    visible: 1,
                    explored: 1,
                }),
                transform: Transform::from_translation(coord_to_vec3(position))
                    .with_rotation(Quat::from_rotation_y(1.0))
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            },
            CampBundle {
                camp,
                view_radius: ViewRadius(VIEW_RADIUS),
                ..default()
            },
        ))
        .id()
}

#[allow(clippy::type_complexity)]
pub fn update_camp_view_radius(
    mut camp_query: Query<(&Group, &mut ViewRadius), (With<Camp>, Changed<Group>)>,
) {
    for (group, mut view_radius) in &mut camp_query {
        view_radius.0 = if group.members.is_empty() {
            0
        } else {
            VIEW_RADIUS
        };
    }
}
