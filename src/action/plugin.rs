use super::{queue::*, system::*};
use crate::{
    actor::SlideEvent,
    turn::{set_player_turn, TurnState},
};
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionUpdate;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        let game_action_systems = GameActionSystems::builder(app.world_mut())
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
        let game_action_follow_up_system =
            GameActionFollowUpSystem(app.world_mut().register_system(follow_up_action));
        app.insert_resource(GameActionQueue::default())
            .insert_resource(game_action_follow_up_system)
            .insert_resource(game_action_systems)
            .init_schedule(ActionUpdate)
            .add_systems(
                Update,
                (
                    (
                        apply_action.map(bevy::utils::warn).run_if(has_ready_action),
                        handle_slide_stopped.run_if(on_event::<SlideEvent>()),
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
