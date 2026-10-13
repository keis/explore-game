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
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_mod_outline::OutlineVolume;
use expl_map::{MapPosition, MapPresence};

pub fn map_position_added(trigger: On<Add, MapPosition>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(handle_zone_click)
        .observe(handle_zone_over.map(error::warn))
        .observe(handle_zone_out.map(error::warn));
}

pub fn selection_added(trigger: On<Add, Selection>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(handle_selection_click)
        .observe(handle_selection_over.map(error::warn))
        .observe(handle_selection_out.map(error::warn));
}

/// Clone initial outline volume when added to be restored after hover cycle.
pub fn outline_volume_added(mut world: DeferredWorld, context: HookContext) {
    let outline_volume = world
        .get_mut::<OutlineVolume>(context.entity)
        .unwrap()
        .clone();
    world
        .commands()
        .entity(context.entity)
        .insert(DefaultOutlineVolume(outline_volume));
}

pub fn apply_zone_activated_event(
    trigger: On<ZoneActivated>,
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
    let target = zone_query.get(trigger.entity)?;
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
    trigger: On<Select>,
    mut selection_query: Query<(&mut Selection, Option<&Children>)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    let Ok((mut selection, children)) = selection_query.get_mut(trigger.entity) else {
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
    trigger: On<Deselect>,
    mut selection_query: Query<(&mut Selection, Option<&Children>)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) {
    let Ok((mut selection, children)) = selection_query.get_mut(trigger.entity) else {
        return;
    };
    selection.is_selected = false;
    for child in children.iter().flat_map(|c| c.iter()) {
        if let Ok((mut outline_volume, default)) = outline_volume_query.get_mut(child) {
            *outline_volume = (*default).clone();
        }
    }
}

fn handle_zone_click(trigger: On<Pointer<Click>>, mut commands: Commands) {
    if trigger.button == PointerButton::Primary {
        commands.trigger(ZoneActivated {
            entity: trigger.entity,
        });
    }
}

fn handle_zone_over(
    trigger: On<Pointer<Over>>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    material_query: Query<&MeshMaterial3d<ZoneMaterial>>,
) -> Result<(), ExplError> {
    let handle = material_query.get(trigger.entity)?;
    let mut material = zone_materials
        .get_mut(handle)
        .ok_or(ExplError::MissingMaterial)?;
    material.set_hover(true);
    Ok(())
}

fn handle_zone_out(
    trigger: On<Pointer<Out>>,
    mut zone_materials: ResMut<Assets<ZoneMaterial>>,
    material_query: Query<&MeshMaterial3d<ZoneMaterial>>,
) -> Result<(), ExplError> {
    let handle = material_query.get(trigger.entity)?;
    let mut material = zone_materials
        .get_mut(handle)
        .ok_or(ExplError::MissingMaterial)?;
    material.set_hover(false);
    Ok(())
}

fn handle_selection_click(trigger: On<Pointer<Click>>, mut selection_update: SelectionUpdate<()>) {
    if trigger.button == PointerButton::Primary {
        selection_update.toggle(trigger.entity);
    }
}

fn handle_selection_over(
    trigger: On<Pointer<Over>>,
    selection_query: Query<&Children, With<Selection>>,
    mut outline_volume_query: Query<&mut OutlineVolume>,
) -> Result<(), ExplError> {
    let children = selection_query.get(trigger.entity)?;
    for child in children.iter() {
        if let Ok(mut outline_volume) = outline_volume_query.get_mut(child) {
            outline_volume.colour = color::OUTLINE_HOVER;
        }
    }
    Ok(())
}

fn handle_selection_out(
    trigger: On<Pointer<Out>>,
    selection_query: Query<(&Selection, &Children)>,
    mut outline_volume_query: Query<(&mut OutlineVolume, &DefaultOutlineVolume)>,
) -> Result<(), ExplError> {
    let (selection, children) = selection_query.get(trigger.entity)?;
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

#[cfg(test)]
mod tests {
    use super::{outline_volume_added, DefaultOutlineVolume};
    use crate::color;
    use bevy::prelude::*;
    use bevy_mod_outline::OutlineVolume;

    #[test]
    fn outline_volume_hook_seeds_default() {
        let mut app = App::new();
        app.world_mut()
            .register_component_hooks::<OutlineVolume>()
            .on_add(outline_volume_added);
        let entity = app
            .world_mut()
            .spawn(OutlineVolume {
                visible: true,
                width: 2.0,
                colour: Color::srgb(1.0, 0.0, 0.0),
            })
            .id();
        app.world_mut().flush();
        let default = app.world().get::<DefaultOutlineVolume>(entity).unwrap();
        assert!(default.0.visible);
        assert_eq!(default.0.width, 2.0);
    }

    #[test]
    fn outline_volume_hover_leaves_default_intact() {
        let mut app = App::new();
        app.world_mut()
            .register_component_hooks::<OutlineVolume>()
            .on_add(outline_volume_added);
        let entity = app
            .world_mut()
            .spawn(OutlineVolume {
                visible: true,
                width: 2.0,
                colour: Color::srgb(1.0, 0.0, 0.0),
            })
            .id();
        app.world_mut().flush();
        app.world_mut()
            .get_mut::<OutlineVolume>(entity)
            .unwrap()
            .colour = color::OUTLINE_HOVER;
        let default = app.world().get::<DefaultOutlineVolume>(entity).unwrap();
        assert_eq!(default.0.colour, Color::srgb(1.0, 0.0, 0.0));
    }
}
