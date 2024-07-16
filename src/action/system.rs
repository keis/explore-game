use super::{plugin::ActionUpdate, queue::*};
use crate::{
    actor::{
        ActorCodex, ActorParams, GroupCommandsExt, Members, Party, PartyBundle, Slide, SlideEvent,
    },
    combat::CombatEvent,
    creature::Movement,
    inventory::Inventory,
    map::{Fog, MapCommandsExt, MapPosition, MapPresence, PresenceLayer, ZoneLayer},
    path::PathGuided,
    role::RoleCommandsExt,
    scene::save,
    structure::{Camp, CampBundle, Portal, SafeHaven, StructureCodex, StructureParams},
    terrain::{CrystalDeposit, HeightQuery, TerrainCodex, TerrainId},
    ExplError,
};
use bevy::prelude::*;
use smallvec::SmallVec;

pub fn apply_action(world: &mut World) -> Result<(), ExplError> {
    let systems = world
        .get_resource::<GameActionSystems>()
        .ok_or(ExplError::ResourceMissing)?;
    let queue = world
        .get_resource::<GameActionQueue>()
        .ok_or(ExplError::ResourceMissing)?;

    let Some(action) = queue.current() else {
        return Ok(());
    };
    let Some(system) = systems.get(action.action_type) else {
        return Ok(());
    };

    let my_action = action.clone();

    let result = world.run_system_with_input(system, my_action.clone())??;

    match result {
        GameActionStatus::Waiting => {
            let mut queue = world
                .get_resource_mut::<GameActionQueue>()
                .ok_or(ExplError::ResourceMissing)?;
            queue.wait();
            return Ok(());
        }
        GameActionStatus::Ready => {
            panic!("Oh no")
        }
        GameActionStatus::Resolved => {}
    }

    world.run_schedule(ActionUpdate);

    let follow_up_system = world
        .get_resource::<GameActionFollowUpSystem>()
        .ok_or(ExplError::ResourceMissing)?;
    let maybe_follow_up: Option<GameAction> =
        world.run_system_with_input(**follow_up_system, my_action.clone())?;

    if let Some(follow_up) = maybe_follow_up {
        let mut queue = world
            .get_resource_mut::<GameActionQueue>()
            .ok_or(ExplError::ResourceMissing)?;
        queue.add(follow_up);
    }

    let mut queue = world
        .get_resource_mut::<GameActionQueue>()
        .ok_or(ExplError::ResourceMissing)?;
    queue.ready();

    Ok(())
}

pub fn resolve_action(world: &mut World) -> Result<(), ExplError> {
    world.run_schedule(ActionUpdate);
    let queue = world
        .get_resource::<GameActionQueue>()
        .ok_or(ExplError::ResourceMissing)?;
    let Some(action) = queue.current() else {
        return Ok(());
    };
    let my_action = action.clone();

    let Some(follow_up_system) = world.get_resource::<GameActionFollowUpSystem>() else {
        return Ok(());
    };
    let maybe_follow_up: Option<GameAction> =
        world.run_system_with_input(**follow_up_system, my_action)?;

    if let Some(follow_up) = maybe_follow_up {
        let mut queue = world
            .get_resource_mut::<GameActionQueue>()
            .ok_or(ExplError::ResourceMissing)?;
        queue.add(follow_up);
    }

    let mut queue = world
        .get_resource_mut::<GameActionQueue>()
        .ok_or(ExplError::ResourceMissing)?;
    queue.ready();

    Ok(())
}

pub fn follow_up_action(
    In(action): In<GameAction>,
    mut combat_events: EventReader<CombatEvent>,
    mut path_guided_query: Query<(&Movement, &mut PathGuided)>,
) -> Option<GameAction> {
    let Ok((party_movement, mut pathguided)) = path_guided_query.get_mut(action.source) else {
        return None;
    };

    pathguided.advance();

    if combat_events.read().count() > 0 {
        return None;
    }

    // Keep moving if a path is set
    if party_movement.current > 0 {
        if let Some(next) = pathguided.next() {
            return Some(GameAction::new_move(action.source, *next));
        }
    }

    None
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_move(
    In(action): In<GameAction>,
    mut commands: Commands,
    mut party_query: Query<
        (
            &mut Slide,
            &mut Transform,
            &MapPresence,
            &mut Movement,
            Option<&Members>,
        ),
        Without<MapPosition>,
    >,
    mut member_movement_query: Query<&mut Movement, Without<MapPresence>>,
    zone_layer_query: Query<(Entity, &ZoneLayer)>,
    map_position_query: Query<(&MapPosition, &Transform)>,
    fog_query: Query<&Fog>,
    height_query: HeightQuery,
) -> GameActionResult {
    let (mut slide, mut transform, presence, mut movement, maybe_members) =
        party_query.get_mut(action.source)?;
    let (map_entity, zone_layer) = zone_layer_query.get_single()?;
    let (next_position, next_transform) = map_position_query.get(action.target()?)?;
    let source_fog = zone_layer
        .get(presence.position)
        .ok_or(ExplError::OutOfBounds)
        .and_then(|&e| fog_query.get(e).map_err(ExplError::from))?;

    movement.consume()?;

    if let Some(members) = maybe_members {
        let mut iter = member_movement_query.iter_many_mut(members.iter());
        while let Some(mut movement) = iter.fetch_next() {
            movement.consume().unwrap();
        }
    }

    if source_fog.visible {
        slide.start = transform.translation;
        slide.end = next_transform.translation;
        slide.progress = 0.0;

        Ok(GameActionStatus::Waiting)
    } else {
        transform.translation = height_query.adjust(next_transform.translation);
        commands
            .entity(map_entity)
            .move_presence(action.source, next_position.0);

        Ok(GameActionStatus::Resolved)
    }
}

pub fn handle_slide_stopped(
    mut commands: Commands,
    mut events: EventReader<SlideEvent>,
    mut queue: ResMut<GameActionQueue>,
    map_query: Query<Entity, With<PresenceLayer>>,
    map_position_query: Query<&MapPosition>,
) {
    let Ok(map_entity) = map_query.get_single() else {
        return;
    };
    for _ in events.read() {
        let Some(action) = queue.current() else {
            return;
        };
        let Ok(target) = action.target() else {
            return;
        };
        let Ok(next) = map_position_query.get(target) else {
            return;
        };

        commands
            .entity(map_entity)
            .move_presence(action.source, next.0);
        queue.resolve();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_make_camp(
    In(action): In<GameAction>,
    mut commands: Commands,
    mut structure_params: StructureParams,
    map_query: Query<(Entity, &ZoneLayer, &PresenceLayer)>,
    terrain_query: Query<&TerrainId>,
    mut party_query: Query<(&mut Inventory, &Members, &MapPresence), With<Party>>,
    camp_query: Query<&Camp>,
    terrain_codex: TerrainCodex,
    structure_codex: StructureCodex,
) -> GameActionResult {
    let terrain_codex = terrain_codex.get()?;
    let structure_codex = structure_codex.get()?;

    let (mut party_inventory, members, presence) = party_query.get_mut(action.source)?;
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
            let (camp_bundle, structure_role) =
                CampBundle::new(position, String::from("New camp"), camp_inventory)
                    .with_fluff(&mut structure_params, structure_codex);
            location
                .spawn((Name::new("Camp"), save::Save, camp_bundle))
                .attach_role(structure_role)
                .add_members(members);
        });
    Ok(GameActionStatus::Resolved)
}

pub fn handle_break_camp(
    In(action): In<GameAction>,
    mut commands: Commands,
    mut party_query: Query<(&mut Inventory, &MapPresence), With<Party>>,
    map_query: Query<(Entity, &PresenceLayer)>,
    camp_query: Query<(Entity, &Members), With<Camp>>,
) -> GameActionResult {
    let (mut inventory, presence) = party_query.get_mut(action.source)?;
    let (map_entity, presence_layer) = map_query.get_single()?;
    let (camp_entity, members) = camp_query
        .iter_many(presence_layer.presence(presence.position))
        .next()
        .ok_or(ExplError::InvalidLocation("no camp".to_string()))?;
    if !members.is_empty() {
        return Err(ExplError::InvalidLocation("camp is not empty".to_string()));
    }
    info!("Depawning camp at {}", presence.position);
    inventory.add_item(Inventory::SUPPLY, 1);
    commands.entity(map_entity).despawn_presence(camp_entity);
    Ok(GameActionStatus::Resolved)
}

#[allow(clippy::type_complexity)]
pub fn handle_enter_camp(
    In(action): In<GameAction>,
    mut commands: Commands,
    mut party_query: Query<(&mut Inventory, &Members), (With<Party>, Without<Camp>)>,
    mut camp_query: Query<&mut Inventory, (With<Camp>, Without<Party>)>,
) -> GameActionResult {
    let (mut party_inventory, members) = party_query.get_mut(action.source)?;
    let camp_entity = action.target()?;
    let mut camp_inventory = camp_query.get_mut(camp_entity)?;

    camp_inventory.take_all(&mut party_inventory);
    commands.entity(camp_entity).add_members(members);

    Ok(GameActionStatus::Resolved)
}

pub fn handle_create_party_from_camp(
    In(action): In<GameAction>,
    mut commands: Commands,
    mut party_params: ActorParams,
    mut camp_query: Query<(&mut Inventory, &MapPresence), With<Camp>>,
    map_query: Query<Entity, With<PresenceLayer>>,
    actor_codex: ActorCodex,
) -> GameActionResult {
    let actor_codex = actor_codex.get()?;
    info!(
        "Creating party at camp {:?} {:?}",
        action.source, action.targets
    );
    let (mut camp_inventory, presence) = camp_query.get_mut(action.source)?;
    let map_entity = map_query.get_single()?;

    let new_supplies = camp_inventory.take_item(Inventory::SUPPLY, 1).unwrap_or(0);

    commands
        .entity(map_entity)
        .with_presence(presence.position, |location| {
            let (party_bundle, party_role, actor_role) =
                PartyBundle::new(presence.position, "New Party".to_string(), new_supplies)
                    .with_fluff(&mut party_params, actor_codex);
            location
                .spawn((Name::new("Party"), save::Save, party_bundle))
                .attach_role(party_role)
                .attach_role(actor_role)
                .add_members(&action.targets);
        });

    Ok(GameActionStatus::Resolved)
}

pub fn handle_split_party(
    In(action): In<GameAction>,
    mut commands: Commands,
    mut party_params: ActorParams,
    mut party_query: Query<(&mut Inventory, &Members, &MapPresence), With<Party>>,
    map_query: Query<Entity, With<PresenceLayer>>,
    actor_codex: ActorCodex,
) -> GameActionResult {
    let actor_codex = actor_codex.get()?;
    let (mut party_inventory, members, presence) = party_query.get_mut(action.source)?;
    let map_entity = map_query.get_single()?;
    if members.len() == action.targets.len() {
        return Err(ExplError::InvalidPartySplit);
    }

    let new_supplies = party_inventory.split_item(Inventory::SUPPLY);

    commands
        .entity(map_entity)
        .with_presence(presence.position, |location| {
            let (party_bundle, party_role, actor_role) =
                PartyBundle::new(presence.position, "New Party".to_string(), new_supplies)
                    .with_fluff(&mut party_params, actor_codex);
            location
                .spawn((Name::new("Party"), save::Save, party_bundle))
                .attach_role(party_role)
                .attach_role(actor_role)
                .add_members(&action.targets);
        });
    Ok(GameActionStatus::Resolved)
}

pub fn handle_merge_party(
    In(action): In<GameAction>,
    mut commands: Commands,
    mut party_query: Query<(&mut Inventory, &Members, &MapPresence), With<Party>>,
) -> GameActionResult {
    let source_position = party_query.get(action.source)?.2.position;
    let mut characters = SmallVec::<[Entity; 8]>::new();
    let mut inventory = Inventory::default();
    let mut iter = party_query.iter_many_mut(&action.targets);
    while let Some((mut party_inventory, members, presence)) = iter.fetch_next() {
        if presence.position != source_position {
            info!("Skipping party in other location");
            continue;
        }
        inventory.take_all(&mut party_inventory);
        characters.append(&mut members.0.clone());
    }
    let (mut party_inventory, _, _) = party_query.get_mut(action.target()?)?;
    party_inventory.take_all(&mut inventory);
    commands.entity(action.source).add_members(&characters);

    Ok(GameActionStatus::Resolved)
}

pub fn handle_collect_crystals(
    In(action): In<GameAction>,
    map_query: Query<&ZoneLayer>,
    mut party_query: Query<(&mut Inventory, &MapPresence), With<Party>>,
    mut crystal_deposit_query: Query<&mut CrystalDeposit>,
) -> GameActionResult {
    let (mut inventory, presence) = party_query.get_mut(action.source)?;
    let zone_layer = map_query.get_single()?;

    let mut crystal_deposit = zone_layer
        .get(presence.position)
        .ok_or(ExplError::OutOfBounds)
        .and_then(|&e| crystal_deposit_query.get_mut(e).map_err(ExplError::from))?;

    inventory.add_item(Inventory::CRYSTAL, crystal_deposit.take() as u32);

    Ok(GameActionStatus::Resolved)
}

pub fn handle_open_portal(
    In(action): In<GameAction>,
    party_query: Query<&MapPresence, With<Party>>,
    map_query: Query<&PresenceLayer>,
    mut portal_query: Query<&mut Portal>,
) -> GameActionResult {
    let presence = party_query.get(action.source)?;
    let presence_layer = map_query.get_single()?;

    let mut portal_iter = portal_query.iter_many_mut(presence_layer.presence(presence.position));
    let mut portal = portal_iter
        .fetch_next()
        .ok_or(ExplError::InvalidLocation("no portal present".to_string()))?;

    if !portal.open {
        portal.open = true;
    }

    Ok(GameActionStatus::Resolved)
}

#[allow(clippy::type_complexity)]
pub fn handle_enter_portal(
    In(action): In<GameAction>,
    mut commands: Commands,
    map_query: Query<&PresenceLayer>,
    portal_query: Query<&Portal>,
    mut party_query: Query<(&MapPresence, &Members, &mut Inventory), With<Party>>,
    mut safe_haven_query: Query<(Entity, &mut Inventory), (With<SafeHaven>, Without<Party>)>,
) -> GameActionResult {
    let (presence, members, mut party_inventory) = party_query.get_mut(action.source)?;
    let presence_layer = map_query.get_single()?;

    let portal = portal_query
        .iter_many(presence_layer.presence(presence.position))
        .next()
        .ok_or(ExplError::InvalidLocation("no portal present".to_string()))?;

    if !portal.open {
        return Err(ExplError::InvalidLocation("portal not active".to_string()));
    }

    let (safe_haven_entity, mut safe_inventory) = safe_haven_query.get_single_mut()?;
    commands.entity(safe_haven_entity).add_members(members);

    safe_inventory.take_all(&mut party_inventory);

    Ok(GameActionStatus::Resolved)
}
