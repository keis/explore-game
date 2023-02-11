use crate::{
    assets::MainAssets,
    hex::coord_to_vec3,
    hexgrid::HexCoord,
    map::{Offset, ViewRadius},
    VIEW_RADIUS,
};
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;

#[derive(Component, Debug, Default)]
pub struct Camp {
    pub name: String,
}

#[derive(Bundle, Default)]
pub struct CampBundle {
    pub camp: Camp,
    pub pickable_bundle: PickableBundle,
    pub offset: Offset,
    pub view_radius: ViewRadius,
}

pub fn spawn_camp(
    commands: &mut Commands,
    params: &mut ParamSet<(Res<MainAssets>, ResMut<Assets<StandardMaterial>>)>,
    position: HexCoord,
    name: String,
) -> Entity {
    commands
        .spawn((
            PbrBundle {
                mesh: params.p0().tent_mesh.clone(),
                material: params.p1().add(Color::rgb(0.631, 0.596, 0.165).into()),
                transform: Transform::from_translation(coord_to_vec3(position))
                    .with_rotation(Quat::from_rotation_y(1.0)),
                ..default()
            },
            CampBundle {
                camp: Camp { name },
                view_radius: ViewRadius(VIEW_RADIUS),
                ..default()
            },
        ))
        .id()
}
