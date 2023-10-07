use super::{
    color::{NORMAL, SELECTED},
    databinding::{DataBindingExt, DataBindings},
    stat::spawn_stat_display,
    InterfaceAssets,
};
use crate::{
    actor::{Character, Group},
    combat::{Attack, Health},
    input::{Action, ActionState, Selection},
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::Pickable;

#[derive(Component)]
pub struct CharacterList;

#[derive(Component)]
pub struct CharacterDisplay {
    character: Entity,
}

#[derive(Bundle)]
pub struct CharacterListBundle {
    node_bundle: NodeBundle,
    character_list: CharacterList,
    pickable: Pickable,
}

#[derive(Component)]
pub struct AttackText;

#[derive(Component)]
pub struct HealthText;

impl Default for CharacterListBundle {
    fn default() -> Self {
        Self {
            node_bundle: NodeBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            character_list: CharacterList,
            pickable: Pickable::IGNORE,
        }
    }
}

fn spawn_character_display(
    parent: &mut ChildBuilder,
    entity: Entity,
    character: &Character,
    attack: &Attack,
    health: &Health,
    assets: &Res<InterfaceAssets>,
) {
    parent
        .spawn((
            CharacterDisplay { character: entity },
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    margin: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                background_color: NORMAL.into(),
                ..default()
            },
        ))
        .bind_to(entity)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                character.name.clone(),
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 28.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn(NodeBundle::default()).with_children(|parent| {
                spawn_stat_display(
                    parent,
                    assets,
                    entity,
                    AttackText,
                    assets.gladius_icon.clone(),
                    format!("{}-{}", attack.low, attack.high),
                );
                spawn_stat_display(
                    parent,
                    assets,
                    entity,
                    HealthText,
                    assets.heart_shield_icon.clone(),
                    format!("{}", health.0),
                );
            });
        });
}

#[allow(clippy::type_complexity)]
pub fn run_if_any_party_or_selection_changed(
    party_query: Query<Entity, Or<(Changed<Group>, Changed<Selection>)>>,
) -> bool {
    !party_query.is_empty()
}

pub fn update_character_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    character_list_query: Query<Entity, With<CharacterList>>,
    character_query: Query<(Entity, &Character, &Attack, &Health)>,
    party_query: Query<(&Group, &Selection), Without<Character>>,
    character_display_query: Query<(Entity, &CharacterDisplay)>,
    mut selection_query: Query<&mut Selection, With<Character>>,
) {
    let character_list = character_list_query.single();

    let characters = party_query
        .iter()
        .filter(|(_, selection)| selection.is_selected)
        .flat_map(|(group, _)| group.members.iter());
    for (entity, character, attack, health) in character_query.iter_many(characters) {
        if !character_display_query
            .iter()
            .any(|(_, display)| display.character == entity)
        {
            commands
                .get_or_spawn(character_list)
                .with_children(|parent| {
                    spawn_character_display(parent, entity, character, attack, health, &assets);
                });
        }
    }

    let characters: Vec<&Entity> = party_query
        .iter()
        .filter(|(_, selection)| selection.is_selected)
        .flat_map(|(party, _)| party.members.iter())
        .collect();
    for (display_entity, display) in character_display_query.iter() {
        if !characters
            .iter()
            .any(|entity| display.character == **entity)
        {
            commands.entity(display_entity).despawn_recursive();
            if let Ok(mut character_selection) = selection_query.get_mut(display.character) {
                character_selection.is_selected = false;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_character_selection(
    mut character_display_query: Query<&mut BackgroundColor, With<CharacterDisplay>>,
    selection_query: Query<(&Selection, &DataBindings), (With<Character>, Changed<Selection>)>,
) {
    for (selection, bindings) in &selection_query {
        let mut character_display_iter = character_display_query.iter_many_mut(bindings);
        while let Some(mut background_color) = character_display_iter.fetch_next() {
            *background_color = if selection.is_selected {
                SELECTED
            } else {
                NORMAL
            }
            .into();
        }
    }
}

pub fn update_character_health(
    mut health_text_query: Query<&mut Text, With<HealthText>>,
    health_query: Query<(&Health, &DataBindings), Changed<Health>>,
) {
    for (health, bindings) in &health_query {
        let mut health_text_iter = health_text_query.iter_many_mut(bindings);
        while let Some(mut text) = health_text_iter.fetch_next() {
            text.sections[0].value = format!("{}", health.0);
        }
    }
}

pub fn handle_character_display_interaction(
    action_state: Res<ActionState<Action>>,
    interaction_query: Query<(&Interaction, &CharacterDisplay), Changed<Interaction>>,
    mut selection_query: Query<(Entity, &mut Selection), With<Character>>,
) {
    if let Ok((Interaction::Pressed, display)) = interaction_query.get_single() {
        if let Ok((entity, mut selection)) = selection_query.get_mut(display.character) {
            if action_state.pressed(Action::MultiSelect) {
                let selected = selection.is_selected;
                selection.is_selected = !selected;
            } else {
                for (e, mut selection) in selection_query.iter_mut() {
                    selection.is_selected = e == entity;
                }
            }
        }
    }
}
