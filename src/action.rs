use crate::camp::Camp;
use crate::hex::coord_to_vec3;
use crate::map::{find_path, MapComponent, MapPresence, PathGuided};
use crate::party::Party;
use crate::HexCoord;
use crate::MainAssets;
use crate::Terrain;
use crate::Zone;
use crate::VIEW_RADIUS;
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;

#[derive(Debug)]
pub enum GameAction {
    MoveTo(Entity, HexCoord),
    MakeCamp(Entity),
    BreakCamp(Entity),
}

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameAction>()
            .add_system(handle_move_to)
            .add_system(handle_make_camp)
            .add_system(handle_break_camp);
    }
}

pub fn handle_move_to(
    mut events: EventReader<GameAction>,
    mut presence_query: Query<(&mut PathGuided, &MapPresence)>,
    zone_query: Query<&Zone>,
    map_query: Query<&MapComponent>,
) {
    // Use let_chains after rust 1.64
    for event in events.iter() {
        if let GameAction::MoveTo(e, goal) = event {
            if let Ok((mut pathguided, presence)) = presence_query.get_mut(*e) {
                if let Ok(map) = map_query.get(presence.map) {
                    if let Some((path, _length)) =
                        find_path(presence.position, *goal, &|c: &HexCoord| {
                            if let Some(entity) = map.storage.get(*c) {
                                if let Ok(zone) = zone_query.get(entity) {
                                    return zone.terrain != Terrain::Lava;
                                }
                            }
                            false
                        })
                    {
                        pathguided.path(path);
                    }
                }
            }
        }
    }
}

pub fn handle_make_camp(
    mut commands: Commands,
    assets: Res<MainAssets>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<GameAction>,
    mut map_query: Query<&mut MapComponent>,
    mut party_query: Query<(&mut Party, &MapPresence)>,
    camp_query: Query<&Camp>,
) {
    for event in events.iter() {
        if let GameAction::MakeCamp(e) = event {
            if let Ok((mut party, presence)) = party_query.get_mut(*e) {
                let mut map = map_query
                    .get_mut(presence.map)
                    .expect("references valid map");

                let position = presence.position;
                if camp_query
                    .iter_many(map.storage.presence(position))
                    .next()
                    .is_some()
                {
                    info!("There's already a camp here");
                    return;
                }

                if party.supplies == 0 {
                    info!("Party does not have enough supplies to make camp");
                    return;
                }

                party.supplies -= 1;
                let entity = commands
                    .spawn_bundle(PbrBundle {
                        mesh: assets.tent_mesh.clone(),
                        material: standard_materials.add(Color::rgb(0.631, 0.596, 0.165).into()),
                        transform: Transform::from_translation(coord_to_vec3(position, 1.0))
                            .with_rotation(Quat::from_rotation_y(1.0)),
                        ..default()
                    })
                    .insert_bundle(PickableBundle::default())
                    .insert(Camp {
                        name: String::from("New Camp"),
                    })
                    .insert(MapPresence {
                        map: presence.map,
                        position,
                        offset: Vec3::ZERO,
                        view_radius: VIEW_RADIUS,
                    })
                    .id();
                map.storage.add_presence(position, entity);
            }
        }
    }
}

pub fn handle_break_camp(
    mut commands: Commands,
    mut events: EventReader<GameAction>,
    mut party_query: Query<(&mut Party, &MapPresence)>,
    mut map_query: Query<&mut MapComponent>,
    camp_query: Query<Entity, With<Camp>>,
) {
    for event in events.iter() {
        if let GameAction::BreakCamp(e) = event {
            if let Ok((mut party, presence)) = party_query.get_mut(*e) {
                let mut map = map_query
                    .get_mut(presence.map)
                    .expect("references valid map");

                let position = presence.position;
                let maybe_camp = camp_query.iter_many(map.storage.presence(position)).next();
                if let Some(camp) = maybe_camp {
                    party.supplies += 1;
                    commands.entity(camp).despawn();
                    map.storage.remove_presence(position, camp);
                }
            }
        }
    }
}
