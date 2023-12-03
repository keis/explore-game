use super::queue::{GameAction, GameActionQueue};
use crate::{
    actor::{
        Group, GroupCommandsExt, Movement, Party, PartyBundle, PartyParams, Slide, SlideEvent,
    },
    combat::CombatEvent,
    inventory::Inventory,
    map::{Fog, MapCommandsExt, MapPresence, Offset, PresenceLayer, ZoneLayer},
    path::{PathFinder, PathGuided},
    scene::save,
    structure::{Camp, CampBundle, CampParams, Portal, SafeHaven},
    terrain::{CrystalDeposit, HeightQuery, TerrainCodex, TerrainId},
    ExplError,
};
use bevy::prelude::*;
use smallvec::SmallVec;

#[allow(clippy::type_complexity)]
pub fn handle_move(
    mut commands: Commands,
    mut queue: ResMut<GameActionQueue>,
    mut party_query: Query<(
        &mut Slide,
        &mut Transform,
        &MapPresence,
        &Offset,
        Option<(&mut Movement, &Group)>,
    )>,
    mut member_movement_query: Query<&mut Movement, Without<MapPresence>>,
    zone_layer_query: Query<(Entity, &ZoneLayer)>,
    fog_query: Query<&Fog>,
    height_query: HeightQuery,
) -> Result<(), ExplError> {
    let Some(GameAction::Move(entity, next)) = queue.current else {
        return Ok(());
    };

    let (mut slide, mut transform, presence, offset, maybe_movement) =
        party_query.get_mut(entity)?;
    let (map_entity, zone_layer) = zone_layer_query.get_single()?;
    let source_fog = zone_layer
        .get(presence.position)
        .ok_or(ExplError::OutOfBounds)
        .and_then(|&e| fog_query.get(e).map_err(ExplError::from))?;

    // Movement is not tracked for enemies
    if let Some((mut movement, group)) = maybe_movement {
        if movement.points == 0 {
            queue.clear();
            return Err(ExplError::MoveWithoutMovementPoints);
        }
        movement.points -= 1;
        let mut iter = member_movement_query.iter_many_mut(&group.members);
        while let Some(mut movement) = iter.fetch_next() {
            movement.points -= 1;
        }
    }

    if source_fog.visible {
        slide.start = transform.translation;
        slide.end = Vec3::from(next) + offset.0;
        slide.progress = 0.0;

        queue.wait();
    } else {
        transform.translation = height_query.adjust(next.into()) + offset.0;
        commands.entity(map_entity).move_presence(entity, next);
    }

    Ok(())
}

pub fn handle_slide_stopped(
    mut commands: Commands,
    mut events: EventReader<SlideEvent>,
    mut queue: ResMut<GameActionQueue>,
    map_query: Query<Entity, With<PresenceLayer>>,
) {
    let Ok(map_entity) = map_query.get_single() else {
        return;
    };
    for _ in events.read() {
        let Some(GameAction::Move(e, next)) = queue.current else {
            return;
        };
        queue.done();

        commands.entity(map_entity).move_presence(e, next);
    }
}

pub fn follow_path(
    mut queue: ResMut<GameActionQueue>,
    mut combat_events: EventReader<CombatEvent>,
    mut path_guided_query: Query<(&Movement, &mut PathGuided)>,
) {
    let Some(GameAction::Move(e, ..)) = queue.current else {
        return;
    };
    let Ok((party_movement, mut pathguided)) = path_guided_query.get_mut(e) else {
        return;
    };

    pathguided.advance();

    if combat_events.read().count() > 0 {
        return;
    }

    // Keep moving if a path is set
    if party_movement.points > 0 {
        if let Some(next) = pathguided.next() {
            queue.add(GameAction::Move(e, *next));
        }
    }
}

pub fn handle_move_to(
    mut queue: ResMut<GameActionQueue>,
    mut presence_query: Query<(&MapPresence, &Movement, &mut PathGuided)>,
    path_finder: PathFinder,
) -> Result<(), ExplError> {
    let Some(GameAction::MoveTo(e, goal)) = queue.current else {
        return Ok(());
    };

    let (presence, party_movement, mut pathguided) = presence_query.get_mut(e)?;
    let Some((path, _length)) = path_finder.get()?.find_path(presence.position, goal) else {
        return Ok(());
    };
    pathguided.path(path);
    if party_movement.points > 0 {
        if let Some(next) = pathguided.next() {
            queue.add(GameAction::Move(e, *next));
        }
    }
    Ok(())
}

pub fn handle_resume_move(
    mut queue: ResMut<GameActionQueue>,
    path_guided_query: Query<&PathGuided>,
) -> Result<(), ExplError> {
    let Some(GameAction::ResumeMove(e)) = queue.current else {
        return Ok(());
    };

    let pathguided = path_guided_query.get(e)?;

    if let Some(next) = pathguided.next() {
        queue.add(GameAction::Move(e, *next));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn handle_make_camp(
    queue: ResMut<GameActionQueue>,
    mut commands: Commands,
    mut spawn_camp_params: CampParams,
    map_query: Query<(Entity, &ZoneLayer, &PresenceLayer)>,
    terrain_query: Query<&TerrainId>,
    mut party_query: Query<(&mut Inventory, &Group, &MapPresence), With<Party>>,
    camp_query: Query<&Camp>,
    terrain_codex: TerrainCodex,
) -> Result<(), ExplError> {
    let Some(GameAction::MakeCamp(party_entity)) = queue.current else {
        return Ok(());
    };
    let terrain_codex = terrain_codex.get()?;

    let (mut party_inventory, group, presence) = party_query.get_mut(party_entity)?;
    let (map_entity, zone_layer, presence_layer) = map_query.get_single()?;
    let terrain_id = zone_layer
        .get(presence.position)
        .ok_or(ExplError::OutOfBounds)
        .and_then(|&e| terrain_query.get(e).map_err(ExplError::from))?;

    if !terrain_codex[terrain_id].allow_structure {
        return Err(ExplError::InvalidLocation("bad terrain".to_string()));
    }

    let position = presence.position;
    if camp_query
        .iter_many(presence_layer.presence(position))
        .next()
        .is_some()
    {
        return Err(ExplError::InvalidLocation("existing camp".to_string()));
    }

    party_inventory.take_item(Inventory::SUPPLY, 1)?;

    let camp_inventory: Inventory = party_inventory.clone();

    info!("Spawning camp at {}", position);
    commands
        .entity(map_entity)
        .with_presence(position, |location| {
            location
                .spawn((
                    Name::new("Camp"),
                    save::Save,
                    CampBundle::new(position, String::from("New camp"), camp_inventory)
                        .with_fluff(&mut spawn_camp_params),
                ))
                .add_members(&group.members);
        });
    Ok(())
}

pub fn handle_break_camp(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_query: Query<(&mut Inventory, &MapPresence), With<Party>>,
    map_query: Query<(Entity, &PresenceLayer)>,
    camp_query: Query<(Entity, &Group), With<Camp>>,
) -> Result<(), ExplError> {
    let Some(GameAction::BreakCamp(e)) = queue.current else {
        return Ok(());
    };

    let (mut inventory, presence) = party_query.get_mut(e)?;
    let (map_entity, presence_layer) = map_query.get_single()?;
    let (camp_entity, group) = camp_query
        .iter_many(presence_layer.presence(presence.position))
        .next()
        .ok_or(ExplError::InvalidLocation("no camp".to_string()))?;
    if !group.members.is_empty() {
        return Err(ExplError::InvalidLocation("camp is not empty".to_string()));
    }
    info!("Depawning camp at {}", presence.position);
    inventory.add_item(Inventory::SUPPLY, 1);
    commands.entity(map_entity).despawn_presence(camp_entity);
    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn handle_enter_camp(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_query: Query<(&mut Inventory, &Group), (With<Party>, Without<Camp>)>,
    mut camp_query: Query<&mut Inventory, (With<Camp>, Without<Party>)>,
) -> Result<(), ExplError> {
    let Some(GameAction::EnterCamp(party_entity, camp_entity)) = queue.current else {
        return Ok(());
    };

    let (mut party_inventory, group) = party_query.get_mut(party_entity)?;
    let mut camp_inventory = camp_query.get_mut(camp_entity)?;

    camp_inventory.take_all(&mut party_inventory);
    commands.entity(camp_entity).add_members(&group.members);

    Ok(())
}

pub fn handle_create_party_from_camp(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_params: PartyParams,
    mut camp_query: Query<(&mut Inventory, &MapPresence), With<Camp>>,
    map_query: Query<Entity, With<PresenceLayer>>,
) -> Result<(), ExplError> {
    let Some(GameAction::CreatePartyFromCamp(camp_entity, characters)) = &queue.current else {
        return Ok(());
    };

    info!("Creating party at camp {:?} {:?}", camp_entity, characters);
    let (mut camp_inventory, presence) = camp_query.get_mut(*camp_entity).unwrap();
    let map_entity = map_query.get_single()?;

    let new_supplies = camp_inventory.take_item(Inventory::SUPPLY, 1).unwrap_or(0);

    commands
        .entity(map_entity)
        .with_presence(presence.position, |location| {
            location
                .spawn((
                    Name::new("Party"),
                    save::Save,
                    PartyBundle::new(presence.position, "New Party".to_string(), new_supplies)
                        .with_fluff(&mut party_params),
                ))
                .add_members(characters);
        });

    Ok(())
}

pub fn handle_split_party(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_params: PartyParams,
    mut party_query: Query<(&mut Inventory, &Group, &MapPresence), With<Party>>,
    map_query: Query<Entity, With<PresenceLayer>>,
) -> Result<(), ExplError> {
    let Some(GameAction::SplitParty(party_entity, characters)) = &queue.current else {
        return Ok(());
    };

    let (mut party_inventory, group, presence) = party_query.get_mut(*party_entity).unwrap();
    let map_entity = map_query.get_single()?;
    if group.members.len() == characters.len() {
        return Err(ExplError::InvalidPartySplit);
    }

    let new_supplies = party_inventory.split_item(Inventory::SUPPLY);

    commands
        .entity(map_entity)
        .with_presence(presence.position, |location| {
            location
                .spawn((
                    Name::new("Party"),
                    save::Save,
                    PartyBundle::new(presence.position, "New Party".to_string(), new_supplies)
                        .with_fluff(&mut party_params),
                ))
                .add_members(characters);
        });
    Ok(())
}

pub fn handle_merge_party(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    mut party_query: Query<(&mut Inventory, &Group, &MapPresence), With<Party>>,
) {
    let Some(GameAction::MergeParty(parties)) = &queue.current else {
        return;
    };

    let [target, rest @ ..] = parties.as_slice() else {
        return;
    };
    let target_position = party_query
        .get(*target)
        .map(|(_, _, p)| p.position)
        .unwrap();
    let mut characters = SmallVec::<[Entity; 8]>::new();
    let mut inventory = Inventory::default();
    let mut iter = party_query.iter_many_mut(rest);
    while let Some((mut party_inventory, group, presence)) = iter.fetch_next() {
        if presence.position != target_position {
            info!("Skipping party in other location");
            continue;
        }
        inventory.take_all(&mut party_inventory);
        characters.append(&mut group.members.clone());
    }
    let (mut party_inventory, _, _) = party_query.get_mut(*target).unwrap();
    party_inventory.take_all(&mut inventory);
    commands.entity(*target).add_members(&characters);
}

pub fn handle_collect_crystals(
    queue: ResMut<GameActionQueue>,
    map_query: Query<&ZoneLayer>,
    mut party_query: Query<(&mut Inventory, &MapPresence), With<Party>>,
    mut crystal_deposit_query: Query<&mut CrystalDeposit>,
) -> Result<(), ExplError> {
    let Some(GameAction::CollectCrystals(party_entity)) = queue.current else {
        return Ok(());
    };

    let (mut inventory, presence) = party_query.get_mut(party_entity)?;
    let zone_layer = map_query.get_single()?;

    let mut crystal_deposit = zone_layer
        .get(presence.position)
        .ok_or(ExplError::OutOfBounds)
        .and_then(|&e| crystal_deposit_query.get_mut(e).map_err(ExplError::from))?;

    inventory.add_item(Inventory::CRYSTAL, crystal_deposit.take() as u32);
    Ok(())
}

pub fn handle_open_portal(
    queue: ResMut<GameActionQueue>,
    party_query: Query<&MapPresence, With<Party>>,
    map_query: Query<&PresenceLayer>,
    mut portal_query: Query<&mut Portal>,
) -> Result<(), ExplError> {
    let Some(GameAction::OpenPortal(party_entity)) = queue.current else {
        return Ok(());
    };

    let presence = party_query.get(party_entity)?;
    let presence_layer = map_query.get_single()?;

    let mut portal_iter = portal_query.iter_many_mut(presence_layer.presence(presence.position));
    let mut portal = portal_iter
        .fetch_next()
        .ok_or(ExplError::InvalidLocation("no portal present".to_string()))?;

    if !portal.open {
        portal.open = true;
    }

    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn handle_enter_portal(
    mut commands: Commands,
    queue: ResMut<GameActionQueue>,
    map_query: Query<&PresenceLayer>,
    portal_query: Query<&Portal>,
    mut party_query: Query<(&MapPresence, &Group, &mut Inventory), With<Party>>,
    mut safe_haven_query: Query<(Entity, &mut Inventory), (With<SafeHaven>, Without<Party>)>,
) -> Result<(), ExplError> {
    let Some(GameAction::EnterPortal(party_entity)) = queue.current else {
        return Ok(());
    };

    let (presence, group, mut party_inventory) = party_query.get_mut(party_entity)?;
    let presence_layer = map_query.get_single()?;

    let portal = portal_query
        .iter_many(presence_layer.presence(presence.position))
        .next()
        .ok_or(ExplError::InvalidLocation("no portal present".to_string()))?;

    if !portal.open {
        return Err(ExplError::InvalidLocation("portal not active".to_string()));
    }

    let (safe_haven_entity, mut safe_inventory) = safe_haven_query.get_single_mut()?;
    commands
        .entity(safe_haven_entity)
        .add_members(&group.members);

    safe_inventory.take_all(&mut party_inventory);

    Ok(())
}
