use bevy::prelude::*;

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct Turn {
    pub number: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum TurnState {
    #[default]
    System,
    Player,
}

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<TurnState>()
            .insert_resource(Turn { number: 0 })
            .add_systems(OnEnter(TurnState::Player), update_turn_counter);
    }
}

pub fn set_system_turn(mut turn_state: ResMut<NextState<TurnState>>) {
    turn_state.set(TurnState::System)
}

pub fn set_player_turn(mut turn_state: ResMut<NextState<TurnState>>) {
    turn_state.set(TurnState::Player)
}

fn update_turn_counter(mut turn: ResMut<Turn>) {
    **turn += 1;
}
