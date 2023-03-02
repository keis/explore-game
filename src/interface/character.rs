use super::{
    color::{NORMAL, SELECTED},
    stat::spawn_stat_display,
    InterfaceAssets,
};
use crate::{
    character::Character,
    combat::{Attack, Health},
    input::{Action, ActionState},
    party::Group,
};
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_mod_picking::Selection;

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
                    size: Size::new(Val::Px(200.0), Val::Auto),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            character_list: CharacterList,
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
                    size: Size::new(Val::Percent(100.0), Val::Px(60.0)),
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
                    AttackText,
                    assets.gladius_icon.clone(),
                    format!("{}-{}", attack.0.start, attack.0.end),
                );
                spawn_stat_display(
                    parent,
                    assets,
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
) -> ShouldRun {
    if !party_query.is_empty() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
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
        .filter(|(_, selection)| selection.selected())
        .flat_map(|(group, _)| group.members.iter());
    for (entity, character, attack, health) in character_query.iter_many(characters) {
        if !character_display_query
            .iter()
            .any(|(_, display)| display.character == entity)
        {
            commands
                .get_or_spawn(character_list)
                .add_children(|parent| {
                    spawn_character_display(parent, entity, character, attack, health, &assets);
                });
        }
    }

    let characters: Vec<&Entity> = party_query
        .iter()
        .filter(|(_, selection)| selection.selected())
        .flat_map(|(party, _)| party.members.iter())
        .collect();
    for (display_entity, display) in character_display_query.iter() {
        if !characters
            .iter()
            .any(|entity| display.character == **entity)
        {
            commands.entity(display_entity).despawn_recursive();
            if let Ok(mut character_selection) = selection_query.get_mut(display.character) {
                character_selection.set_selected(false);
            }
        }
    }
}

pub fn update_character_selection(
    mut character_display_query: Query<(&CharacterDisplay, &mut BackgroundColor)>,
    selection_query: Query<&Selection, (With<Character>, Changed<Selection>)>,
) {
    for (display, mut color) in character_display_query.iter_mut() {
        if let Ok(selection) = selection_query.get(display.character) {
            if selection.selected() {
                *color = SELECTED.into();
            } else {
                *color = NORMAL.into();
            }
        }
    }
}

pub fn update_character_health(
    mut health_text_query: Query<(&mut Text, &Parent), With<HealthText>>,
    intermediate_parent_query: Query<&Parent>,
    character_display_query: Query<&CharacterDisplay>,
    health_query: Query<&Health, Changed<Health>>,
) {
    for (mut text, parent) in health_text_query.iter_mut() {
        let Ok(intermediate_parent_a) = intermediate_parent_query.get(parent.get()) else { continue };
        let Ok(intermediate_parent_b) = intermediate_parent_query.get(intermediate_parent_a.get()) else { continue };
        let Ok(character_display) = character_display_query.get(intermediate_parent_b.get()) else { continue };
        let Ok(health) = health_query.get(character_display.character) else { continue };
        text.sections[0].value = format!("{}", health.0);
    }
}

pub fn handle_character_display_interaction(
    action_state_query: Query<&ActionState<Action>>,
    interaction_query: Query<(&Interaction, &CharacterDisplay), Changed<Interaction>>,
    mut selection_query: Query<(Entity, &mut Selection), With<Character>>,
) {
    let action_state = action_state_query.single();
    if let Ok((Interaction::Clicked, display)) = interaction_query.get_single() {
        if let Ok((entity, mut selection)) = selection_query.get_mut(display.character) {
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
