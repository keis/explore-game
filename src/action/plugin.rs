use super::{component::*, event::*, queue::*, system::*};
use crate::{
    actor::SlideEvent,
    scene::SceneState,
    turn::{set_player_turn, TurnState},
};
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionUpdate;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        let game_action_systems = GameActions::builder(app.world_mut())
            .register_action(GameActionType::Move, ActionCost::World, handle_move)
            .register_action(
                GameActionType::MakeCamp,
                ActionCost::World,
                handle_make_camp,
            )
            .register_action(
                GameActionType::BreakCamp,
                ActionCost::World,
                handle_break_camp,
            )
            .register_action(
                GameActionType::EnterCamp,
                ActionCost::Free,
                handle_enter_camp,
            )
            .register_action(
                GameActionType::SplitParty,
                ActionCost::Free,
                handle_split_party,
            )
            .register_action(
                GameActionType::MergeParty,
                ActionCost::Free,
                handle_merge_party,
            )
            .register_action(
                GameActionType::CreatePartyFromCamp,
                ActionCost::Free,
                handle_create_party_from_camp,
            )
            .register_action(
                GameActionType::CollectCrystals,
                ActionCost::World,
                handle_collect_crystals,
            )
            .register_action(
                GameActionType::OpenPortal,
                ActionCost::World,
                handle_open_portal,
            )
            .register_action(
                GameActionType::EnterPortal,
                ActionCost::World,
                handle_enter_portal,
            )
            .build();
        let game_action_follow_up_system =
            GameActionFollowUpSystem(app.world_mut().register_system(follow_up_action));
        app.insert_resource(GameActionQueue::default())
            .insert_resource(game_action_follow_up_system)
            .insert_resource(game_action_systems)
            .add_event::<ActionPointsConsumed>()
            .init_schedule(ActionUpdate)
            .register_type::<ActionPoints>()
            .add_observer(update_action_points_on_member_added)
            .add_observer(update_action_points_on_member_removed)
            .add_observer(propagate_action_points_consumed)
            .add_systems(
                OnEnter(TurnState::Player),
                (reset_action_points, reset_group_action_points)
                    .chain()
                    .run_if(in_state(SceneState::Active)),
            )
            .add_systems(
                Update,
                (
                    (
                        apply_action.map(bevy::utils::warn).run_if(has_ready_action),
                        handle_slide_stopped.run_if(on_event::<SlideEvent>),
                        resolve_action
                            .map(bevy::utils::warn)
                            .run_if(has_resolved_action),
                    )
                        .chain(),
                    set_player_turn
                        .run_if(in_state(TurnState::System))
                        .run_if(action_queue_is_empty),
                ),
            );
    }
}
