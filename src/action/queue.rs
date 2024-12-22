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

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameActionStatus {
    #[default]
    Ready,
    Waiting,
    Resolved,
}

pub type GameActionResult = Result<GameActionStatus, ExplError>;

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
    deque: VecDeque<GameAction>,
    status: GameActionStatus,
}

impl GameActionQueue {
    pub fn add(&mut self, action: GameAction) {
        self.deque.push_back(action);
    }

    pub fn is_waiting(&self) -> bool {
        self.status == GameActionStatus::Waiting
    }

    pub fn is_resolved(&self) -> bool {
        self.status == GameActionStatus::Resolved
    }

    pub fn wait(&mut self) {
        self.status = GameActionStatus::Waiting;
    }

    pub fn resolve(&mut self) {
        self.status = GameActionStatus::Resolved;
    }

    pub fn ready(&mut self) {
        self.status = GameActionStatus::Ready;
        self.deque.pop_front();
    }

    pub fn current(&self) -> Option<&GameAction> {
        self.deque.front()
    }

    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }
}

#[derive(Clone, Copy)]
pub enum ActionCost {
    Free,
    World,
}

pub struct GameActionInfo {
    pub(super) system: SystemId<In<GameAction>, GameActionResult>,
    pub(super) action_cost: ActionCost,
}

#[derive(Resource)]
pub struct GameActions(EnumMap<GameActionType, Option<GameActionInfo>>);

impl GameActions {
    pub fn builder(world: &mut World) -> GameActionSystemsBuilder {
        GameActionSystemsBuilder::from_world(world)
    }

    pub fn get(&self, action: GameActionType) -> Option<&GameActionInfo> {
        self.0[action].as_ref()
    }
}

pub struct GameActionSystemsBuilder<'a> {
    world: &'a mut World,
    enum_map: EnumMap<GameActionType, Option<GameActionInfo>>,
}

impl<'a> GameActionSystemsBuilder<'a> {
    fn from_world(world: &'a mut World) -> Self {
        Self {
            world,
            enum_map: EnumMap::default(),
        }
    }

    pub fn register_action<F, Marker>(
        mut self,
        action: GameActionType,
        action_point_cost: ActionCost,
        f: F,
    ) -> Self
    where
        F: IntoSystem<In<GameAction>, GameActionResult, Marker> + 'static,
    {
        self.enum_map[action] = Some(GameActionInfo {
            system: self.world.register_system(f),
            action_cost: action_point_cost,
        });
        self
    }

    pub fn build(self) -> GameActions {
        GameActions(self.enum_map)
    }
}

#[derive(Resource, Deref)]
pub struct GameActionFollowUpSystem(pub(super) SystemId<In<GameAction>, Option<GameAction>>);

pub fn has_ready_action(
    game_action_queue: Res<GameActionQueue>,
    combat_query: Query<&Combat>,
) -> bool {
    !game_action_queue.is_waiting()
        && game_action_queue.current().is_some()
        && combat_query.is_empty()
}

pub fn has_resolved_action(game_action_queue: Res<GameActionQueue>) -> bool {
    game_action_queue.is_resolved()
}

pub fn action_queue_is_empty(game_action_queue: Res<GameActionQueue>) -> bool {
    game_action_queue.is_empty()
}
