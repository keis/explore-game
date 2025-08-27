use super::{component::*, event::*, system_param::*};
use crate::{
    action::{ActionPoints, GameAction, GameActionQueue},
    color,
    combat::Combat,
    error,
    material::ZoneMaterial,
    path::{PathFinder, PathGuided},
    terrain::TerrainId,
    ExplError,
};
use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;
use expl_map::{MapPosition, MapPresence};

pub fn map_position_added(trigger: Trigger<OnAdd, MapPosition>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .observe(handle_zone_click)
        .observe(handle_zone_over.map(error::warn))
        .observe(handle_zone_out.map(error::warn));
}

pub fn selection_added(trigger: Trigger<OnAdd, Selection>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .observe(handle_selection_click)
        .observe(handle_selection_over.map(error::warn))
        .observe(handle_selection_out.map(error::warn));
}

pub fn apply_zone_activated_event(
    trigger: Trigger<ZoneActivated>,
    mut presence_query: Query<(
        Entity,
        &MapPresence,
        &ActionPoints,
        &mut PathGuided,
        &Selection,
    )>,
    mut game_action_queue: ResMut<GameActionQueue>,
    zone_query: Query<Entity, With<TerrainId>>,
    combat_query: Query<&Combat>,
    map_position_query: Query<&MapPosition>,
    path_finder: PathFinder,
) -> Result<(), ExplError> {
    if !combat_query.is_empty() {
        return Ok(());
    }
    let target = zone_query.get(trigger.target())?;
    for (entity, presence, action_points, mut pathguided, _) in presence_query
        .iter_mut()
        .filter(|(_, _, _, _, s)| s.is_selected)
    {
        let goal = map_position_query.get(target)?;
        let path_finder = path_finder.get()?;
        let Some(path) = path_finder.find_path(presence.position, goal.0) else {
            continue;
        };
        pathguided.path(path.into_iter().map(|(_, e)| e));
        if action_points.current > 0 {
            if let Some(next) = pathguided.next() {
                game_action_queue.add(GameAction::new_move(entity, *next));
            }
        }
    }
    Ok(())
}

pub fn apply_select_event(
    trigger: Trigger<Select>,
    mut selection_query: Query<(&mut Selection, Option<&Children>)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    let Ok((mut selection, children)) = selection_query.get_mut(trigger.target()) else {
        return;
    };
    selection.is_selected = true;
    for child in children.iter().flat_map(|c| c.iter()) {
        if let Ok((mut outline_volume, _)) = outline_volume_query.get_mut(child) {
            outline_volume.colour = color::OUTLINE_SELECTED;
        }
    }
}

pub fn apply_deselect_event(
    trigger: Trigger<Deselect>,
    mut selection_query: Query<(&mut Selection, Option<&Children>)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    let Ok((mut selection, children)) = selection_query.get_mut(trigger.target()) else {
        return;
    };
    selection.is_selected = false;
    for child in children.iter().flat_map(|c| c.iter()) {
        if let Ok((mut outline_volume, default)) = outline_volume_query.get_mut(child) {
            *outline_volume = (*default).clone();
        }
    }
}

fn handle_zone_click(trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    if trigger.event().button == PointerButton::Primary {
        commands.trigger_targets(ZoneActivated, trigger.target());
    }
}

fn handle_zone_over(
    trigger: Trigger<Pointer<Over>>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    material_query: Query<&MeshMaterial3d<ZoneMaterial>>,
) -> Result<(), ExplError> {
    let handle = material_query.get(trigger.target())?;
    let material = zone_materials
        .get_mut(handle)
        .ok_or(ExplError::MissingMaterial)?;
    material.set_hover(true);
    Ok(())
}

fn handle_zone_out(
    trigger: Trigger<Pointer<Out>>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    material_query: Query<&MeshMaterial3d<ZoneMaterial>>,
) -> Result<(), ExplError> {
    let handle = material_query.get(trigger.target())?;
    let material = zone_materials
        .get_mut(handle)
        .ok_or(ExplError::MissingMaterial)?;
    material.set_hover(false);
    Ok(())
}

fn handle_selection_click(
    trigger: Trigger<Pointer<Click>>,
    mut selection_update: SelectionUpdate<()>,
) {
    if trigger.event().button == PointerButton::Primary {
        selection_update.toggle(trigger.target());
    }
}

fn handle_selection_over(
    trigger: Trigger<Pointer<Over>>,
    selection_query: Query<&Children, With<Selection>>,
    mut outline_volume_query: Query<&mut OutlineVolume>,
) -> Result<(), ExplError> {
    let children = selection_query.get(trigger.target())?;
    for child in children.iter() {
        if let Ok(mut outline_volume) = outline_volume_query.get_mut(child) {
            outline_volume.colour = color::OUTLINE_HOVER;
        }
    }
    Ok(())
}

fn handle_selection_out(
    trigger: Trigger<Pointer<Out>>,
    selection_query: Query<(&Selection, &Children)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) -> Result<(), ExplError> {
    let (selection, children) = selection_query.get(trigger.target())?;
    for child in children.iter() {
        if let Ok((mut outline_volume, default)) = outline_volume_query.get_mut(child) {
            if selection.is_selected {
                outline_volume.colour = Color::srgb(0.75, 0.50, 0.50);
            } else {
                *outline_volume = (*default).clone();
            };
        }
    }
    Ok(())
}
