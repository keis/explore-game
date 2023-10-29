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
        app.insert_resource(GameActionQueue::default())
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
                    handle_move.pipe(warn).run_if(has_current_action),
                    handle_move_to.pipe(warn).run_if(has_current_action),
                    handle_resume_move.pipe(warn).run_if(has_current_action),
                    handle_make_camp.pipe(warn).run_if(has_current_action),
                    handle_break_camp.pipe(warn).run_if(has_current_action),
                    handle_enter_camp.pipe(warn).run_if(has_current_action),
                    handle_create_party_from_camp
                        .pipe(warn)
                        .run_if(has_current_action),
                    handle_split_party.pipe(warn).run_if(has_current_action),
                    handle_merge_party.run_if(has_current_action),
                    handle_collect_crystals
                        .pipe(warn)
                        .run_if(has_current_action),
                    handle_open_portal.pipe(warn).run_if(has_current_action),
                    handle_enter_portal.pipe(warn).run_if(has_current_action),
                    handle_slide_stopped
                        .run_if(on_event::<SlideEvent>())
                        .after(handle_move),
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
