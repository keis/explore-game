//! Game-specific BRP methods for agent-driven testing.
//!
//! Stock BRP covers generic ECS operations, but game flows (menu buttons,
//! picking-based selection, state transitions) go through states and
//! observers that generic methods can't reach. These methods replay the
//! exact code paths the UI would trigger. Only registered when the `brp`
//! feature is enabled.
use std::collections::BTreeMap;

use crate::{input::Select, interface::InterfaceState, scene::SceneState};
use bevy::{
    ecs::reflect::AppTypeRegistry,
    prelude::*,
    reflect::{PartialReflect, TypeInfo, TypeRegistry},
    remote::{error_codes::INVALID_PARAMS, BrpResult, RemoteMethodSystemId, RemoteMethods},
};
#[cfg(feature = "brp")]
use bevy_brp_extras::{AgentTool, AppAgentToolExt};
use schemars::JsonSchema;
use serde::{de::DeserializeSeed as _, Deserialize, Serialize};
use serde_json::{json, Value};

/// Registers the `game/*` methods. Must run after `RemotePlugin` (added by
/// `BrpExtrasPlugin`) so the `RemoteMethods` resource exists.
pub struct DebugBrpPlugin;

impl Plugin for DebugBrpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_debug_methods);
        #[cfg(feature = "brp")]
        {
            app.register_agent_tool(AgentTool::new(
                "game_new_game",
                "game/new_game",
                "Replicates the New game menu button: back to shell UI and regenerate the world",
            ));
            app.register_agent_tool(
                AgentTool::new(
                    "game_select",
                    "game/select",
                    "Replicates clicking an entity by triggering Select on it",
                )
                .params_schema_for::<SelectParams>(),
            );
            app.register_agent_tool(
                AgentTool::new(
                    "game_set_state",
                    "game/set_state",
                    "Queues transitions for any registered game state (e.g. InterfaceState, SceneState) by short name or full path. Multiple states are queued in one call and applied together at the next state-transition point, running the normal OnExit/OnEnter schedules",
                )
                .params_schema_for::<SetStateParams>()
                .result_schema_for::<SetStateResult>(),
            );
        }
    }
}

/// Raw JSON-RPC `params` for `game/select`: `{"entity": <u64 id>}`.
///
/// Also serves as the published parameter schema for the agent tool, so the
/// documented shape and the parsed shape cannot drift apart.
#[derive(Deserialize, JsonSchema)]
struct SelectParams {
    entity: u64,
}

fn register_debug_methods(world: &mut World) {
    let new_game = world.register_system(new_game_handler);
    let select = world.register_system(select_handler);
    let set_state = world.register_system(set_state_handler);
    let mut methods = world.resource_mut::<RemoteMethods>();
    methods.insert("game/new_game", RemoteMethodSystemId::Instant(new_game));
    methods.insert("game/select", RemoteMethodSystemId::Instant(select));
    methods.insert("game/set_state", RemoteMethodSystemId::Instant(set_state));
}

/// Replicates the New game menu button: back to shell UI, regenerate world.
fn new_game_handler(
    _: In<Option<Value>>,
    mut next_scene_state: ResMut<NextState<SceneState>>,
    mut next_interface_state: ResMut<NextState<InterfaceState>>,
) -> BrpResult {
    next_interface_state.set(InterfaceState::Shell);
    next_scene_state.set(SceneState::Reset);
    Ok(Value::Null)
}

/// Replicates clicking an entity: `{"entity": 4294967298}`.
fn select_handler(In(params): In<Option<Value>>, mut commands: Commands) -> BrpResult {
    let entity: Entity = params
        .as_ref()
        .and_then(|p| serde_json::from_value::<SelectParams>(p.clone()).ok())
        .and_then(|p| Entity::try_from_bits(p.entity))
        .ok_or_else(|| bevy::remote::BrpError {
            code: INVALID_PARAMS,
            message: "expected {\"entity\": <u64 id>}".to_string(),
            data: None,
        })?;
    commands.trigger(Select { entity });
    Ok(Value::Null)
}

/// Raw JSON-RPC `params` for `game/set_state`:
/// `{"updates": {"InterfaceState": "Shell", "SceneState": "Reset"}}`.
///
/// Keys accept a state type's full path or its short name when unambiguous;
/// values are variant names. Works for any state type the app reflects and
/// registers alongside its `NextState` (see the `register_type` calls in the
/// scene/interface plugins). All entries are validated first, then queued in
/// one run and applied together at the next state-transition point.
#[derive(Deserialize, JsonSchema)]
struct SetStateParams {
    updates: BTreeMap<String, String>,
}

/// Raw JSON-RPC `result` for `game/set_state`: the queued transitions.
#[derive(Serialize, JsonSchema)]
struct SetStateResult {
    applied: BTreeMap<String, String>,
}

fn set_state_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let updates = params
        .as_ref()
        .map(|p| serde_json::from_value::<SetStateParams>(p.clone()))
        .transpose()
        .map_err(|err| {
            invalid_params(format!(
                "expected {{\"updates\": {{\"State\": \"Variant\", ...}}}}: {err}"
            ))
        })?
        .ok_or_else(|| invalid_params("missing parameters"))?
        .updates;
    if updates.is_empty() {
        return Err(invalid_params("`updates` must not be empty"));
    }

    let registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = registry.read();

    // Resolve every entry before touching the world so one bad entry rejects
    // the whole request instead of queueing a partial transition.
    let mut staged = Vec::with_capacity(updates.len());
    for (state, variant) in &updates {
        let (resource_id, value) = resolve_state_transition(&type_registry, world, state, variant)?;
        staged.push((resource_id, value));
    }
    drop(type_registry);

    for (resource_id, value) in staged {
        world.insert_reflect_resource(resource_id, value);
    }

    serde_json::to_value(SetStateResult { applied: updates }).map_err(|err| {
        bevy::remote::BrpError::internal(format!("failed to serialize result: {err}"))
    })
}

/// Builds the `NextState::Pending` value for one `state -> variant` pair and
/// returns it with the resource id to insert it as.
fn resolve_state_transition(
    type_registry: &TypeRegistry,
    world: &World,
    state_key: &str,
    variant: &str,
) -> Result<(bevy::ecs::component::ComponentId, Box<dyn PartialReflect>), bevy::remote::BrpError> {
    let state_registration = if state_key.contains("::") {
        type_registry.get_with_type_path(state_key)
    } else {
        type_registry.get_with_short_type_path(state_key)
    }
    .ok_or_else(|| {
        invalid_params(format!(
            "unknown state `{state_key}`; expected a registered state type (short name or full path). Available states: {}",
            available_states(type_registry)
        ))
    })?;

    let TypeInfo::Enum(state_info) = state_registration.type_info() else {
        return Err(invalid_params(format!("`{state_key}` is not a state enum")));
    };
    if !state_info.contains_variant(variant) {
        return Err(invalid_params(format!(
            "unknown variant `{variant}` for state `{state_key}`; expected one of: {}",
            state_info.variant_names().join(", ")
        )));
    }

    let next_path = format!(
        "bevy_state::state::resources::NextState<{}>",
        state_registration.type_info().type_path()
    );
    let next_registration =
        type_registry
            .get_with_type_path(&next_path)
            .ok_or_else(|| {
                invalid_params(format!(
                    "`{next_path}` is not registered; register it with `.register_type::<NextState<...>>()`"
                ))
            })?;

    let value =
        bevy::reflect::serde::TypedReflectDeserializer::new(next_registration, type_registry)
            .deserialize(&json!({ "Pending": variant }))
            .map_err(|err| {
                invalid_params(format!(
                    "invalid transition {state_key} -> {variant}: {err}"
                ))
            })?;

    let resource_id = world
        .components()
        .get_id(next_registration.type_id())
        .ok_or_else(|| {
            invalid_params(format!(
                "`{next_path}` is registered but not present in the world; is its state initialized?"
            ))
        })?;

    Ok((resource_id, value))
}

/// Short paths of every registered `NextState`, e.g. `NextState<SceneState>`.
fn available_states(type_registry: &TypeRegistry) -> String {
    let mut states: Vec<&str> = type_registry
        .iter()
        .map(|registration| registration.type_info().type_path_table().short_path())
        .filter(|path| path.starts_with("NextState<"))
        .collect();
    states.sort_unstable();
    states.join(", ")
}

fn invalid_params(message: impl ToString) -> bevy::remote::BrpError {
    bevy::remote::BrpError {
        code: INVALID_PARAMS,
        message: message.to_string(),
        data: None,
    }
}
