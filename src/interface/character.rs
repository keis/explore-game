use super::{
    color::{NORMAL, SELECTED},
    InterfaceAssets,
};
use crate::{
    character::Character,
    input::{Action, ActionState},
    party::Party,
};
use bevy::prelude::*;
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

impl Default for CharacterListBundle {
    fn default() -> Self {
        Self {
            node_bundle: NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
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
    assets: &Res<InterfaceAssets>,
) {
    parent
        .spawn((
            CharacterDisplay { character: entity },
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(40.0)),
                    margin: UiRect::all(Val::Px(2.0)),
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
        });
}

pub fn update_character_list(
    mut commands: Commands,
    assets: Res<InterfaceAssets>,
    character_list_query: Query<Entity, With<CharacterList>>,
    character_query: Query<(Entity, &Character)>,
    party_query: Query<(&Party, &Selection), Without<Character>>,
    character_display_query: Query<(Entity, &CharacterDisplay)>,
    mut selection_query: Query<&mut Selection, With<Character>>,
) {
    let character_list = character_list_query.single();

    let characters = party_query
        .iter()
        .filter(|(_, selection)| selection.selected())
        .flat_map(|(party, _)| party.members.iter());
    for (entity, character) in character_query.iter_many(characters) {
        if !character_display_query
            .iter()
            .any(|(_, display)| display.character == entity)
        {
            commands
                .get_or_spawn(character_list)
                .add_children(|parent| {
                    spawn_character_display(parent, entity, character, &assets);
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
