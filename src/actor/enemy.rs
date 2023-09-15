use crate::{
    action::{GameAction, GameActionQueue},
    actor::{party::Party, slide::Slide},
    assets::MainAssets,
    combat::{Attack, Health},
    map::{HexCoord, MapPresence, Offset, PathFinder, PresenceLayer, ViewRadius, ZoneLayer},
    terrain::{HeightQuery, Terrain},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Enemy;

pub type EnemyParams<'w, 's> = (
    Res<'w, MainAssets>,
    ResMut<'w, Assets<StandardMaterial>>,
    HeightQuery<'w, 's>,
);

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    enemy: Enemy,
    offset: Offset,
    presence: MapPresence,
    view_radius: ViewRadius,
    slide: Slide,
    attack: Attack,
    health: Health,
}

#[derive(Bundle, Default)]
pub struct EnemyFluffBundle {
    pbr_bundle: PbrBundle,
    outline_bundle: OutlineBundle,
}

impl EnemyBundle {
    pub fn new(position: HexCoord) -> Self {
        let presence = MapPresence { position };
        let offset = Offset(Vec3::new(0.0, 0.05, 0.0));
        Self {
            offset,
            presence,
            view_radius: ViewRadius(3),
            attack: Attack { low: 1, high: 10 },
            health: Health(20, 20),
            ..default()
        }
    }

    pub fn with_fluff(self, enemy_params: &mut EnemyParams) -> (Self, EnemyFluffBundle) {
        let fluff = EnemyFluffBundle::new(enemy_params, &self.presence, &self.offset);
        (self, fluff)
    }
}

impl EnemyFluffBundle {
    pub fn new(
        (main_assets, standard_materials, height_query): &mut EnemyParams,
        presence: &MapPresence,
        offset: &Offset,
    ) -> Self {
        Self {
            pbr_bundle: PbrBundle {
                mesh: main_assets.blob_mesh.clone(),
                material: standard_materials.add(Color::rgba(0.749, 0.584, 0.901, 0.666).into()),
                transform: Transform::from_translation(
                    height_query.adjust(presence.position.into()) + offset.0,
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
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn fluff_enemy(
    mut commands: Commands,
    enemy_query: Query<(Entity, &MapPresence, &Offset), (With<Enemy>, Without<GlobalTransform>)>,
    mut enemy_params: EnemyParams,
) {
    for (entity, presence, offset) in &enemy_query {
        commands
            .entity(entity)
            .insert(EnemyFluffBundle::new(&mut enemy_params, presence, offset));
    }
}

#[derive(SystemParam)]
pub struct Target<'w, 's> {
    presence_query: Query<'w, 's, &'static MapPresence, With<Party>>,
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
    map_query: Query<(&PresenceLayer, &ZoneLayer)>,
    enemy_query: Query<(Entity, &MapPresence, &ViewRadius), With<Enemy>>,
    terrain_query: Query<&Terrain>,
    target: Target,
    path_finder: PathFinder,
) {
    let Ok((presence_layer, zone_layer)) = map_query.get_single() else { return };
    let mut rng = thread_rng();
    for (entity, presence, view_radius) in &enemy_query {
        if let Some(target) = target.closest_in_view(presence.position, view_radius) {
            let Some((path, _length)) = path_finder.find_path(presence.position, target.position) else { continue };
            let Some(next) = path.get(1) else { continue };
            if enemy_query
                .iter_many(presence_layer.presence(*next))
                .next()
                .is_some()
            {
                continue;
            }
            queue.add(GameAction::Move(entity, *next));
        } else {
            let mut neighbours = HexCoord::NEIGHBOUR_OFFSETS;
            neighbours.shuffle(&mut rng);
            for offset in neighbours {
                let next = presence.position + offset;
                if zone_layer
                    .get(next)
                    .and_then(|&entity| terrain_query.get(entity).ok())
                    .map_or(false, |terrain| terrain.is_walkable())
                    && enemy_query
                        .iter_many(presence_layer.presence(next))
                        .next()
                        .is_none()
                {
                    queue.add(GameAction::Move(entity, next));
                    break;
                }
            }
        }
    }
}
