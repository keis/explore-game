use crate::{
    action::{GameAction, GameActionQueue},
    assets::MainAssets,
    combat::{Attack, Health},
    map::{HeightQuery, HexCoord, MapPresence, Offset, PathFinder, ViewRadius},
    slide::Slide,
    turn::Turn,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_mod_outline::{OutlineBundle, OutlineVolume};

#[derive(Component, Default)]
pub struct Enemy;

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub offset: Offset,
    pub view_radius: ViewRadius,
    pub slide: Slide,
    pub attack: Attack,
    pub health: Health,
    pub pbr_bundle: PbrBundle,
    pub outline_bundle: OutlineBundle,
}

pub type EnemyParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<StandardMaterial>>,
    HeightQuery<'w, 's>,
);

impl EnemyBundle {
    pub fn new(
        (main_assets, standard_materials, height_query): &mut EnemyParams,
        position: HexCoord,
    ) -> Self {
        let offset = Vec3::new(0.0, 0.05, 0.0);
        Self {
            offset: Offset(offset),
            view_radius: ViewRadius(3),
            attack: Attack(1..10),
            health: Health(20),
            pbr_bundle: PbrBundle {
                mesh: main_assets.blob_mesh.clone(),
                material: standard_materials.add(Color::rgba(0.749, 0.584, 0.901, 0.666).into()),
                transform: Transform::from_translation(
                    height_query.adjust(position.into()) + offset,
                )
                .with_scale(Vec3::splat(0.5)),
                visibility: Visibility::Hidden,
                ..default()
            },
            outline_bundle: OutlineBundle {
                outline: OutlineVolume {
                    visible: true,
                    width: 2.0,
                    colour: Color::rgb(0.739, 0.574, 0.891),
                },
                ..default()
            },
            ..default()
        }
    }
}

#[derive(SystemParam)]
pub struct Target<'w, 's> {
    presence_query: Query<'w, 's, &'static MapPresence, Without<Enemy>>,
}

impl<'w, 's> Target<'w, 's> {
    fn closest_in_view(
        &self,
        position: HexCoord,
        view_radius: &ViewRadius,
    ) -> Option<&MapPresence> {
        self.presence_query
            .iter()
            .filter(|&other| position.distance(other.position) <= view_radius.0)
            .min_by_key(|other| position.distance(other.position))
    }
}

pub fn move_enemy(
    mut queue: ResMut<GameActionQueue>,
    turn: Res<Turn>,
    enemy_query: Query<(Entity, &MapPresence, &ViewRadius), With<Enemy>>,
    target: Target,
    path_finder: PathFinder,
) {
    if turn.is_changed() {
        for (entity, presence, view_radius) in &enemy_query {
            let Some(target) = target.closest_in_view(presence.position, view_radius) else { continue };
            let Some((path, _length)) = path_finder.find_path(presence.position, target.position) else { continue };
            let Some(next) = path.get(1) else { continue };
            queue.add(GameAction::Move(entity, *next));
        }
    }
}
