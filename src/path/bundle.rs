use super::component::*;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;

pub type PathDisplayParams<'w> = (
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<StandardMaterial>>,
);

#[derive(Bundle)]
pub struct PathDisplayBundle {
    path_display: PathDisplay,
    pbr_bundle: PbrBundle,
    not_shadow_caster: NotShadowCaster,
}

impl PathDisplayBundle {
    pub fn new(
        (meshes, standard_materials): &mut PathDisplayParams,
        path_guided: Entity,
        path: Path,
    ) -> Self {
        Self {
            path_display: PathDisplay { path_guided },
            pbr_bundle: PbrBundle {
                mesh: meshes.add(path),
                material: standard_materials.add(Color::srgba(0.8, 0.8, 0.8, 0.6)),
                transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                ..default()
            },
            not_shadow_caster: NotShadowCaster,
        }
    }
}
