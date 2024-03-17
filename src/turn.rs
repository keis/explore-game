use bevy::prelude::*;

#[derive(Resource, Reflect, Copy, Clone, Default, Debug, Deref, DerefMut)]
#[reflect(Resource)]
pub struct Turn {
    pub number: u32,
}

#[derive(Resource, Reflect, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
#[reflect(Resource)]
pub enum Period {
    Morning,
    #[default]
    Day,
    Evening,
    Night,
}

impl From<Turn> for Period {
    #[inline]
    fn from(value: Turn) -> Self {
        match value.number % 4 {
            0 => Period::Day,
            1 => Period::Evening,
            2 => Period::Night,
            3 => Period::Morning,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum TurnState {
    #[default]
    System,
    Player,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum TurnSet {
    Setup,
    Effects,
}

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TurnState>()
            .register_type::<Turn>()
            .insert_resource(Period::default())
            .insert_resource(Turn { number: 0 })
            .configure_sets(
                OnEnter(TurnState::Player),
                (TurnSet::Setup, TurnSet::Effects).chain(),
            )
            .add_systems(
                OnEnter(TurnState::Player),
                (update_turn_counter, apply_period)
                    .chain()
                    .in_set(TurnSet::Setup),
            );
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

fn apply_period(turn: Res<Turn>, mut period_state: ResMut<Period>) {
    *period_state = Period::from(*turn);
}
