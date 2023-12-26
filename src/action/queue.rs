use crate::{combat::Combat, ExplError};
use bevy::{ecs::system::SystemId, prelude::*};
use enum_map::{Enum, EnumMap};
use smallvec::SmallVec;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Enum)]
pub enum GameActionType {
    Move,
    MakeCamp,
    BreakCamp,
    EnterCamp,
    SplitParty,
    MergeParty,
    CreatePartyFromCamp,
    CollectCrystals,
    OpenPortal,
    EnterPortal,
}

#[derive(Clone, Debug)]
pub struct GameAction {
    pub(super) action_type: GameActionType,
    pub(super) source: Entity,
    pub(super) targets: SmallVec<[Entity; 8]>,
}

impl GameAction {
    pub fn target(&self) -> Result<Entity, ExplError> {
        self.targets
            .first()
            .copied()
            .ok_or(ExplError::InvalidTarget)
    }

    pub fn new_move(source: Entity, dest: Entity) -> Self {
        Self {
            action_type: GameActionType::Move,
            source,
            targets: SmallVec::from_slice(&[dest]),
        }
    }

    pub fn new_make_camp(source: Entity) -> Self {
        Self {
            action_type: GameActionType::MakeCamp,
            source,
            targets: SmallVec::default(),
        }
    }

    pub fn new_break_camp(source: Entity) -> Self {
        Self {
            action_type: GameActionType::BreakCamp,
            source,
            targets: SmallVec::default(),
        }
    }

    pub fn new_enter_camp(source: Entity, camp: Entity) -> Self {
        Self {
            action_type: GameActionType::EnterCamp,
            source,
            targets: SmallVec::from_slice(&[camp]),
        }
    }

    pub fn new_split_party<Characters>(source: Entity, characters: Characters) -> Self
    where
        Characters: IntoIterator<Item = Entity>,
    {
        Self {
            action_type: GameActionType::SplitParty,
            source,
            targets: characters.into_iter().collect(),
        }
    }

    pub fn new_merge_party<Parties>(source: Entity, parties: Parties) -> Self
    where
        Parties: IntoIterator<Item = Entity>,
    {
        Self {
            action_type: GameActionType::MergeParty,
            source,
            targets: parties.into_iter().collect(),
        }
    }

    pub fn new_create_party_from_camp<Characters>(source: Entity, characters: Characters) -> Self
    where
        Characters: IntoIterator<Item = Entity>,
    {
        Self {
            action_type: GameActionType::CreatePartyFromCamp,
            source,
            targets: characters.into_iter().collect(),
        }
    }

    pub fn new_collect_crystals(source: Entity) -> Self {
        Self {
            action_type: GameActionType::CollectCrystals,
            source,
            targets: SmallVec::default(),
        }
    }

    pub fn new_open_portal(source: Entity) -> Self {
        Self {
            action_type: GameActionType::OpenPortal,
            source,
            targets: SmallVec::default(),
        }
    }

    pub fn new_enter_portal(source: Entity) -> Self {
        Self {
            action_type: GameActionType::EnterPortal,
            source,
            targets: SmallVec::default(),
        }
    }
}

#[derive(Default, Resource)]
pub struct GameActionQueue {
    pub deque: VecDeque<GameAction>,
    pub current: Option<GameAction>,
    waiting: bool,
}

#[derive(Resource)]
pub struct GameActionSystems(EnumMap<GameActionType, Option<SystemId>>);

impl GameActionSystems {
    pub fn builder(world: &mut World) -> GameActionSystemsBuilder {
        GameActionSystemsBuilder::from_world(world)
    }

    pub fn get(&self, action: GameActionType) -> Option<SystemId> {
        self.0[action]
    }
}

pub struct GameActionSystemsBuilder<'a> {
    world: &'a mut World,
    enum_map: EnumMap<GameActionType, Option<SystemId>>,
}

impl<'a> GameActionSystemsBuilder<'a> {
    fn from_world(world: &'a mut World) -> Self {
        Self {
            world,
            enum_map: EnumMap::default(),
        }
    }

    pub fn register_action<F, Marker>(mut self, action: GameActionType, f: F) -> Self
    where
        F: IntoSystem<(), Result<(), ExplError>, Marker> + 'static,
    {
        self.enum_map[action] = Some(self.world.register_system(f.map(bevy::utils::warn)));
        self
    }

    pub fn build(self) -> GameActionSystems {
        GameActionSystems(self.enum_map)
    }
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
