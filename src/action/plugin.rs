use super::{queue::*, system::*};
use crate::{
    actor::SlideEvent,
    turn::{set_player_turn, TurnState},
};
use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ActionSet {
    Prepare,
    Apply,
    CommandFlush,
    PostApply,
    FollowUp,
    Cleanup,
}

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        let game_action_systems = GameActionSystems::builder(&mut app.world)
            .register_action(GameActionType::Move, handle_move)
            .register_action(GameActionType::MakeCamp, handle_make_camp)
            .register_action(GameActionType::BreakCamp, handle_break_camp)
            .register_action(GameActionType::EnterCamp, handle_enter_camp)
            .register_action(GameActionType::SplitParty, handle_split_party)
            .register_action(GameActionType::MergeParty, handle_merge_party)
            .register_action(
                GameActionType::CreatePartyFromCamp,
                handle_create_party_from_camp,
            )
            .register_action(GameActionType::CollectCrystals, handle_collect_crystals)
            .register_action(GameActionType::OpenPortal, handle_open_portal)
            .register_action(GameActionType::EnterPortal, handle_enter_portal)
            .build();
        app.insert_resource(GameActionQueue::default())
            .insert_resource(game_action_systems)
            .configure_sets(
                Update,
                (
                    ActionSet::Prepare,
                    ActionSet::Apply,
                    ActionSet::CommandFlush,
                    ActionSet::PostApply,
                    ActionSet::FollowUp,
                    ActionSet::Cleanup,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                advance_action_queue
                    .run_if(ready_for_next_action)
                    .in_set(ActionSet::Prepare),
            )
            .add_systems(
                Update,
                (
                    apply_action
                        .map(bevy::utils::warn)
                        .run_if(has_current_action),
                    handle_slide_stopped
                        .run_if(on_event::<SlideEvent>())
                        .after(apply_action),
                )
                    .in_set(ActionSet::Apply),
            )
            .add_systems(
                Update,
                (
                    apply_deferred.in_set(ActionSet::CommandFlush),
                    follow_path
                        .run_if(has_current_action)
                        .in_set(ActionSet::FollowUp),
                    clear_current_action
                        .run_if(has_current_action)
                        .in_set(ActionSet::Cleanup),
                    set_player_turn
                        .run_if(in_state(TurnState::System))
                        .run_if(action_queue_is_empty),
                ),
            );
    }
}
