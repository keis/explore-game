use super::{
    color::{NORMAL, SELECTED},
    databinding::{DataBindingExt, DataBindings},
    stat::spawn_stat_display,
    InterfaceAssets,
};
use crate::{
    camp::Camp,
    character::Character,
    input::{Action, ActionState},
    map::MapEvent,
    party::Group,
};
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_mod_picking::Selection;

#[derive(Component)]
pub struct CampList;

#[derive(Component, Debug)]
pub struct CampDisplay {
    camp: Entity,
}

#[derive(Component)]
pub struct CampCrystalsText;

#[derive(Bundle)]
pub struct CampListBundle {
    node_bundle: NodeBundle,
    camp_list: CampList,
}

impl Default for CampListBundle {
    fn default() -> Self {
        Self {
            node_bundle: NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), Val::Auto),
                    flex_direction: FlexDirection::Column,
                    margin: UiRect {
                        right: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::NONE.into(),
                ..default()
            },
            camp_list: CampList,
        }
    }
}

fn spawn_camp_display(
    parent: &mut ChildBuilder,
    entity: Entity,
    camp: &Camp,
    assets: &Res<InterfaceAssets>,
) {
    parent
        .spawn((
            CampDisplay { camp: entity },
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(120.0)),
                    margin: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                background_color: NORMAL.into(),
                ..default()
            },
        ))
        .bind_to(entity)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                camp.name.clone(),
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn(NodeBundle::default()).with_children(|parent| {
                spawn_stat_display(
                    parent,
                    assets,
                    entity,
                    CampCrystalsText,
                    assets.crystals_icon.clone(),
                    format!("{}", camp.crystals),
                );
            });
        });
}

#[allow(clippy::type_complexity)]
pub fn run_if_any_camp_changed(
    camp_query: Query<Entity, Or<(Changed<Camp>, Changed<Group>)>>,
    mut map_events: EventReader<MapEvent>,
) -> ShouldRun {
    let removed_event_count = map_events
        .iter()
        .filter(|e| matches!(e, MapEvent::PresenceRemoved { .. }))
        .count();
    if camp_query.is_empty() && removed_event_count == 0 {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

pub fn update_camp_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    camp_list_query: Query<Entity, With<CampList>>,
    camp_query: Query<(Entity, &Camp)>,
    camp_display_query: Query<(Entity, &CampDisplay)>,
) {
    let camp_list = camp_list_query.single();
    for (entity, camp) in camp_query.iter() {
        if camp_display_query
            .iter()
            .any(|(_, display)| display.camp == entity)
        {
            continue;
        }
        commands.get_or_spawn(camp_list).add_children(|parent| {
            spawn_camp_display(parent, entity, camp, &assets);
        });
    }

    let camp_entities: Vec<Entity> = camp_query.iter().map(|(e, _)| e).collect();
    for (display_entity, display) in camp_display_query.iter() {
        if !camp_entities.iter().any(|&entity| display.camp == entity) {
            commands.entity(display_entity).despawn_recursive();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_camp_selection(
    mut camp_display_query: Query<&mut BackgroundColor, With<CampDisplay>>,
    selection_query: Query<(&Selection, &DataBindings), (With<Camp>, Changed<Selection>)>,
) {
    for (selection, bindings) in &selection_query {
        let mut camp_display_iter = camp_display_query.iter_many_mut(bindings);
        while let Some(mut background_color) = camp_display_iter.fetch_next() {
            *background_color = if selection.selected() {
                SELECTED
            } else {
                NORMAL
            }
            .into();
        }
    }
}

pub fn update_camp_crystals(
    mut camp_crystals_text_query: Query<&mut Text, With<CampCrystalsText>>,
    camp_query: Query<(&Camp, &DataBindings), Changed<Camp>>,
) {
    for (camp, bindings) in &camp_query {
        let mut camp_crystals_text_iter = camp_crystals_text_query.iter_many_mut(bindings);
        while let Some(mut text) = camp_crystals_text_iter.fetch_next() {
            text.sections[0].value = format!("{}", camp.crystals);
        }
    }
}

pub fn handle_camp_display_interaction(
    action_state_query: Query<&ActionState<Action>>,
    interaction_query: Query<(&Interaction, &CampDisplay), Changed<Interaction>>,
    mut selection_query: Query<(Entity, &mut Selection), Without<Character>>,
) {
    let action_state = action_state_query.single();
    if let Ok((Interaction::Clicked, display)) = interaction_query.get_single() {
        if let Ok((entity, mut selection)) = selection_query.get_mut(display.camp) {
            if action_state.pressed(Action::MultiSelect) {
                let selected = selection.selected();
                selection.set_selected(!selected);
            } else {
                for (e, mut selection) in selection_query.iter_mut() {
                    selection.set_selected(e == entity)
                }
            }
        }
    }
}
