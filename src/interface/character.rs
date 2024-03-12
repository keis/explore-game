use super::{
    color::{NORMAL, SELECTED},
    stat::spawn_stat_display,
    InterfaceAssets,
};
use crate::{
    actor::{Character, Members},
    combat::{Attack, Health},
    input::{Deselect, Selection, SelectionUpdate},
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::Pickable;
use expl_databinding::{DataBindingExt, DataBindingUpdate};

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
                    format!("{}", health.current),
                );
            });
        });
}

pub fn update_character_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    character_list_query: Query<Entity, With<CharacterList>>,
    character_query: Query<(Entity, &Character, &Attack, &Health)>,
    party_query: Query<(&Members, &Selection), Without<Character>>,
    character_display_query: Query<(Entity, &CharacterDisplay)>,
    mut deselect_events: EventWriter<Deselect>,
) {
    let character_list = character_list_query.single();

    let characters = party_query
        .iter()
        .filter(|(_, selection)| selection.is_selected)
        .flat_map(|(members, _)| members.iter());
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
        .flat_map(|(members, _)| members.iter())
        .collect();
    for (display_entity, display) in character_display_query.iter() {
        if !characters
            .iter()
            .any(|entity| display.character == **entity)
        {
            commands.entity(display_entity).despawn_recursive();
            deselect_events.send(Deselect(display.character));
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_character_selection(
    mut data_binding_update: DataBindingUpdate<
        &Selection,
        &mut BackgroundColor,
        (Changed<Selection>, With<Character>),
    >,
) {
    data_binding_update.for_each(|selection, background_color| {
        **background_color = if selection.is_selected {
            SELECTED
        } else {
            NORMAL
        }
        .into();
    });
}

pub fn update_character_health(
    mut data_binding_update: DataBindingUpdate<
        &Health,
        &mut Text,
        (Changed<Health>, With<Character>),
    >,
) {
    data_binding_update.for_each(|health, text| {
        text.sections[0].value = format!("{}", health.current);
    });
}

pub fn handle_character_display_interaction(
    interaction_query: Query<(&Interaction, &CharacterDisplay), Changed<Interaction>>,
    mut selection: SelectionUpdate<With<Character>>,
) {
    if let Ok((Interaction::Pressed, display)) = interaction_query.get_single() {
        selection.toggle(display.character);
    }
}
