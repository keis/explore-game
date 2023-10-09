use crate::{
    actor::Group,
    assets::MainAssets,
    combat::Health,
    input::SelectionBundle,
    map::{Fog, HexCoord, MapPresence, Offset, ViewRadius},
    material::TerrainMaterial,
    VIEW_RADIUS,
};
use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Camp {
    pub name: String,
    pub supplies: u32,
    pub crystals: u32,
}

pub type CampParams<'w> = (Res<'w, MainAssets>, ResMut<'w, Assets<TerrainMaterial>>);

#[derive(Bundle, Default)]
pub struct CampBundle {
    camp: Camp,
    presence: MapPresence,
    group: Group,
    offset: Offset,
    view_radius: ViewRadius,
    fog: Fog,
}

#[derive(Bundle, Default)]
pub struct CampFluffBundle {
    selection: SelectionBundle,
    material_mesh_bundle: MaterialMeshBundle<TerrainMaterial>,
}

impl CampBundle {
    pub fn new(position: HexCoord, camp: Camp) -> Self {
        Self {
            camp,
            presence: MapPresence { position },
            view_radius: ViewRadius(VIEW_RADIUS),
            ..default()
        }
    }

    pub fn with_fluff(self, camp_params: &mut CampParams) -> (Self, CampFluffBundle) {
        let fluff = CampFluffBundle::new(camp_params, &self.presence, &self.offset);
        (self, fluff)
    }
}

impl CampFluffBundle {
    pub fn new(
        (main_assets, terrain_materials): &mut CampParams,
        presence: &MapPresence,
        offset: &Offset,
    ) -> Self {
        Self {
            material_mesh_bundle: MaterialMeshBundle {
                mesh: main_assets.tent_mesh.clone(),
                material: terrain_materials.add(TerrainMaterial {
                    color: Color::rgb(0.631, 0.596, 0.165),
                    visible: true,
                    explored: true,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::from(presence.position) + offset.0)
                    .with_rotation(Quat::from_rotation_y(1.0))
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            },
            ..default()
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn fluff_camp(
    mut commands: Commands,
    mut camp_params: CampParams,
    camp_query: Query<(Entity, &MapPresence, &Offset), (With<Camp>, Without<GlobalTransform>)>,
) {
    for (entity, presence, offset) in &camp_query {
        commands
            .entity(entity)
            .insert(CampFluffBundle::new(&mut camp_params, presence, offset));
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
            VIEW_RADIUS
        };
    }
}

pub fn heal_characters(
    camp_query: Query<&Group, With<Camp>>,
    mut health_query: Query<&mut Health>,
) {
    for group in &camp_query {
        let mut iter = health_query.iter_many_mut(&group.members);
        while let Some(mut health) = iter.fetch_next() {
            health.heal(2);
        }
    }
}
