use crate::input::{Action, ActionState};
use crate::party::Party;
use bevy::prelude::*;
use bevy_mod_picking::Selection;

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_interface)
            .add_system(update_party_list)
            .add_system(update_party_display)
            .add_system(handle_party_display_interaction);
    }
}

#[derive(Component)]
pub struct ZoneText;

#[derive(Component)]
struct PartyList;

#[derive(Component, Debug)]
pub struct PartyDisplay {
    party: Entity,
}

const NORMAL: Color = Color::rgb(0.20, 0.20, 0.20);
const SELECTED: Color = Color::rgb(0.75, 0.50, 0.50);

fn spawn_interface(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(
            TextBundle::from_section(
                "Zone: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            )
            .with_text_alignment(TextAlignment::TOP_CENTER)
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ZoneText);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexEnd,
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .insert(PartyList);
        });
}

fn update_party_list(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    party_list_query: Query<Entity, With<PartyList>>,
    party_query: Query<(Entity, &Party)>,
    party_display_query: Query<&PartyDisplay>,
) {
    let party_list = party_list_query.single();
    for (entity, party) in party_query.iter() {
        if !party_display_query
            .iter()
            .any(|display| display.party == entity)
        {
            commands.get_or_spawn(party_list).add_children(|parent| {
                parent
                    .spawn()
                    .insert(PartyDisplay { party: entity })
                    .insert_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Px(120.0)),
                            margin: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        color: NORMAL.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            party.name.clone(),
                            TextStyle {
                                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                font_size: 32.0,
                                color: Color::WHITE,
                            },
                        ));
                    });
            });
        }
    }
}

pub fn update_party_display(
    mut party_display_query: Query<(&PartyDisplay, &mut UiColor)>,
    party_query: Query<&Selection, (With<Party>, Changed<Selection>)>,
) {
    for (party_display, mut color) in party_display_query.iter_mut() {
        if let Ok(selection) = party_query.get(party_display.party) {
            if selection.selected() {
                *color = SELECTED.into();
            } else {
                *color = NORMAL.into();
            }
        }
    }
}

pub fn handle_party_display_interaction(
    action_state_query: Query<&ActionState<Action>>,
    interaction_query: Query<(&Interaction, &PartyDisplay), Changed<Interaction>>,
    mut party_query: Query<(Entity, &Party, &mut Selection)>,
) {
    let action_state = action_state_query.single();
    if let Ok((Interaction::Clicked, partydisplay)) = interaction_query.get_single() {
        if let Ok((entity, party, mut selection)) = party_query.get_mut(partydisplay.party) {
            info!("Clicked party {:?}", party);
            if action_state.pressed(Action::MultiSelect) {
                let selected = selection.selected();
                selection.set_selected(!selected);
            } else {
                for (e, _, mut selection) in party_query.iter_mut() {
                    selection.set_selected(e == entity)
                }
            }
        }
    }
}
