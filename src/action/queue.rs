use crate::{combat::Combat, map::HexCoord};
use bevy::prelude::*;
use smallvec::SmallVec;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum GameAction {
    Move(Entity, HexCoord),
    MoveTo(Entity, HexCoord),
    ResumeMove(Entity),
    MakeCamp(Entity),
    BreakCamp(Entity),
    EnterCamp(Entity, Entity),
    SplitParty(Entity, SmallVec<[Entity; 8]>),
    MergeParty(SmallVec<[Entity; 8]>),
    CreatePartyFromCamp(Entity, SmallVec<[Entity; 8]>),
    CollectCrystals(Entity),
    OpenPortal(Entity),
}

#[derive(Default, Resource)]
pub struct GameActionQueue {
    pub deque: VecDeque<GameAction>,
    pub current: Option<GameAction>,
    waiting: bool,
}

impl GameActionQueue {
    pub fn add(&mut self, action: GameAction) {
        self.deque.push_back(action);
    }

    pub fn is_waiting(&self) -> bool {
        self.waiting
    }

    pub fn has_next(&self) -> bool {
        !self.deque.is_empty()
    }

    pub fn start_next(&mut self) {
        self.current = self.deque.pop_front();
    }

    pub fn wait(&mut self) {
        self.waiting = true;
    }

    pub fn done(&mut self) {
        self.waiting = false;
    }

    pub fn clear(&mut self) {
        self.current = None;
    }
}

pub fn advance_action_queue(mut game_action_queue: ResMut<GameActionQueue>) {
    game_action_queue.start_next();
}

pub fn clear_current_action(mut game_action_queue: ResMut<GameActionQueue>) {
    game_action_queue.clear();
}

pub fn has_current_action(game_action_queue: Res<GameActionQueue>) -> bool {
    !game_action_queue.is_waiting() && game_action_queue.current.is_some()
}

pub fn action_queue_is_empty(game_action_queue: Res<GameActionQueue>) -> bool {
    game_action_queue.current.is_none() && !game_action_queue.has_next()
}

pub fn ready_for_next_action(
    game_action_queue: Res<GameActionQueue>,
    combat_query: Query<&Combat>,
) -> bool {
    !game_action_queue.is_waiting()
        && (game_action_queue.current.is_some() || game_action_queue.has_next())
        && combat_query.is_empty()
}
