//! Game-specific BRP methods for agent-driven testing.
//!
//! Stock BRP covers generic ECS operations, but game flows (menu buttons,
//! picking-based selection, state transitions) go through states and
//! observers that generic methods can't reach. These methods replay the
//! exact code paths the UI would trigger. Only registered when the `brp`
//! feature is enabled.
use std::collections::{BTreeMap, VecDeque};

use crate::{input::Select, interface::InterfaceState, scene::SceneState};
use bevy::{
    camera::{primitives::Aabb, visibility::InheritedVisibility},
    ecs::{message::Message, reflect::AppTypeRegistry},
    input::mouse::MouseMotion,
    prelude::*,
    reflect::{PartialReflect, TypeInfo, TypeRegistry},
    remote::{error_codes::INVALID_PARAMS, BrpResult, RemoteMethodSystemId, RemoteMethods},
    ui::{CalculatedClip, ComputedNode, UiGlobalTransform},
    window::{CursorMoved, PrimaryWindow, Window, WindowEvent},
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
            app.register_agent_tool(
                AgentTool::new(
                    "game_mouse_over_ui",
                    "game/mouse_over_ui",
                    "Moves the simulated cursor to the center of a UI node, selected by exactly one of entity or name. Returns logical window coordinates. Hover updates on the next frame, so poll Hovered before using brp_extras/click_mouse. Named buttons include Menu/Resume, Toolbar/Camp, Next Turn Button, and Game over/Play again.",
                )
                .params_schema_for::<MouseOverParams>()
                .result_schema_for::<MouseOverUiResult>(),
            );
            app.register_agent_tool(
                AgentTool::new(
                    "game_mouse_over_mesh",
                    "game/mouse_over_mesh",
                    "Moves the simulated cursor to the projected center of a mesh, selected by exactly one of entity or name. Uses the entity's own visible Mesh3d or its first visible mesh descendant, and returns logical window coordinates plus the world point. Hover updates on the next frame, so poll Hovered before using brp_extras/click_mouse.",
                )
                .params_schema_for::<MouseOverParams>()
                .result_schema_for::<MouseOverMeshResult>(),
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

const MOUSE_OVER_PARAMS_HELP: &str =
    "expected exactly one of {\"entity\": <u64 id>} or {\"name\": \"<entity name>\"}";

/// Raw JSON-RPC `params` for `game/mouse_over_ui` and
/// `game/mouse_over_mesh`.
///
/// Exactly one of `entity` or `name` must be provided. Names are matched
/// against the entity's `Name` component and must be unique.
#[derive(Deserialize, JsonSchema)]
struct MouseOverParams {
    /// Entity id (bits) to target.
    #[serde(default)]
    entity: Option<u64>,
    /// Exact entity name to target. Must match exactly one entity.
    #[serde(default)]
    name: Option<String>,
}

/// Raw JSON-RPC `result` for `game/mouse_over_ui`.
#[derive(Serialize, JsonSchema)]
struct MouseOverUiResult {
    /// Entity id of the targeted UI node.
    entity: u64,
    /// Name of the targeted UI node, if it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Logical cursor position inside the primary window.
    position: [f32; 2],
    /// Logical size of the targeted UI node.
    size: [f32; 2],
    /// Entity id of the primary window.
    window: u64,
}

/// Raw JSON-RPC `result` for `game/mouse_over_mesh`.
#[derive(Serialize, JsonSchema)]
struct MouseOverMeshResult {
    /// Entity id of the target used to find the mesh.
    entity: u64,
    /// Name of the target, if it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Entity id of the mesh used to compute the target point.
    mesh_entity: u64,
    /// Logical cursor position inside the primary window.
    position: [f32; 2],
    /// World-space center of the mesh bounds, transformed by its global
    /// transform.
    world_position: [f32; 3],
    /// Entity id of the `Camera3d` used for projection.
    camera: u64,
    /// Entity id of the primary window.
    window: u64,
}

fn register_debug_methods(world: &mut World) {
    let new_game = world.register_system(new_game_handler);
    let select = world.register_system(select_handler);
    let set_state = world.register_system(set_state_handler);
    let mouse_over_ui = world.register_system(mouse_over_ui_handler);
    let mouse_over_mesh = world.register_system(mouse_over_mesh_handler);
    let mut methods = world.resource_mut::<RemoteMethods>();
    methods.insert("game/new_game", RemoteMethodSystemId::Instant(new_game));
    methods.insert("game/select", RemoteMethodSystemId::Instant(select));
    methods.insert("game/set_state", RemoteMethodSystemId::Instant(set_state));
    methods.insert(
        "game/mouse_over_ui",
        RemoteMethodSystemId::Instant(mouse_over_ui),
    );
    methods.insert(
        "game/mouse_over_mesh",
        RemoteMethodSystemId::Instant(mouse_over_mesh),
    );
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

/// Moves the simulated cursor to a UI node's center.
///
/// The target must be laid out and visible. The returned position is in
/// logical window pixels; hover state is updated by the next frame.
fn mouse_over_ui_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let target = resolve_mouse_over_target(world, params)?;
    let (computed, transform) = {
        let computed = world
            .get::<ComputedNode>(target.entity)
            .copied()
            .ok_or_else(|| {
                invalid_params(format!(
                    "entity {} is not a UI node (missing ComputedNode)",
                    target.entity.to_bits()
                ))
            })?;
        let transform = world
            .get::<UiGlobalTransform>(target.entity)
            .copied()
            .ok_or_else(|| {
                invalid_params(format!(
                    "entity {} has no UiGlobalTransform",
                    target.entity.to_bits()
                ))
            })?;
        (computed, transform)
    };

    if computed.size.x <= 0.0 || computed.size.y <= 0.0 || !computed.size.is_finite() {
        return Err(invalid_params(format!(
            "UI node {} has no laid-out area (size {:?})",
            target.entity.to_bits(),
            computed.size
        )));
    }
    if world
        .get::<InheritedVisibility>(target.entity)
        .is_some_and(|visibility| !visibility.get())
    {
        return Err(invalid_params(format!(
            "UI node {} is not visible",
            target.entity.to_bits()
        )));
    }
    if matches!(
        world.get::<CalculatedClip>(target.entity),
        Some(CalculatedClip::FullyClipped)
    ) {
        return Err(invalid_params(format!(
            "UI node {} is fully clipped",
            target.entity.to_bits()
        )));
    }

    // ComputedNode and UiGlobalTransform are in physical pixels; event
    // positions and Window::cursor_position are logical pixels.
    let position = transform.translation * computed.inverse_scale_factor;
    let size = computed.size * computed.inverse_scale_factor;
    if !position.is_finite() {
        return Err(invalid_params(format!(
            "UI node {} has a non-finite layout position",
            target.entity.to_bits()
        )));
    }

    let window = move_simulated_cursor(world, position)?;
    serialize_result(MouseOverUiResult {
        entity: target.entity.to_bits(),
        name: target.name,
        position: position.to_array(),
        size: size.to_array(),
        window: window.to_bits(),
    })
}

/// Moves the simulated cursor to the projected center of a mesh.
///
/// The target's own `Mesh3d` is preferred; otherwise the first visible mesh
/// descendant is used. The returned position is in logical window pixels and
/// hover state is updated by the next frame.
fn mouse_over_mesh_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let target = resolve_mouse_over_target(world, params)?;
    let mesh_entity = find_mesh_entity(world, target.entity).ok_or_else(|| {
        invalid_params(format!(
            "entity {} has no visible Mesh3d with bounds",
            target.entity.to_bits()
        ))
    })?;
    let world_position = {
        let aabb = world.get::<Aabb>(mesh_entity).copied().ok_or_else(|| {
            invalid_params(format!(
                "mesh entity {} has no Aabb bounds",
                mesh_entity.to_bits()
            ))
        })?;
        let transform = world
            .get::<GlobalTransform>(mesh_entity)
            .copied()
            .ok_or_else(|| {
                invalid_params(format!(
                    "mesh entity {} has no GlobalTransform",
                    mesh_entity.to_bits()
                ))
            })?;
        transform.transform_point(Vec3::from(aabb.center))
    };

    let (camera, position) = project_to_viewport(world, world_position)?;
    let window = move_simulated_cursor(world, position)?;
    serialize_result(MouseOverMeshResult {
        entity: target.entity.to_bits(),
        name: target.name,
        mesh_entity: mesh_entity.to_bits(),
        position: position.to_array(),
        world_position: world_position.to_array(),
        camera: camera.to_bits(),
        window: window.to_bits(),
    })
}

fn resolve_mouse_over_target(
    world: &mut World,
    params: Option<Value>,
) -> Result<ResolvedTarget, bevy::remote::BrpError> {
    let params = params
        .map(serde_json::from_value::<MouseOverParams>)
        .transpose()
        .map_err(|err| invalid_params(format!("{MOUSE_OVER_PARAMS_HELP}: {err}")))?
        .ok_or_else(|| invalid_params(format!("missing parameters; {MOUSE_OVER_PARAMS_HELP}")))?;

    let entity = match (params.entity, params.name) {
        (Some(id), None) => Entity::try_from_bits(id)
            .filter(|entity| world.get_entity(*entity).is_ok())
            .ok_or_else(|| invalid_params(format!("no entity with id {id}")))?,
        (None, Some(name)) => {
            if name.is_empty() {
                return Err(invalid_params("`name` must not be empty"));
            }
            let matches = {
                let mut query = world.query::<(Entity, &Name)>();
                query
                    .iter(world)
                    .filter(|(_, entity_name)| entity_name.as_str() == name)
                    .map(|(entity, _)| entity)
                    .collect::<Vec<_>>()
            };
            match matches.as_slice() {
                [entity] => *entity,
                [] => return Err(invalid_params(format!("no entity named `{name}`"))),
                _ => {
                    let ids = matches
                        .iter()
                        .map(|entity| entity.to_bits().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(invalid_params(format!(
                        "name `{name}` matches multiple entities ({ids}); pass `entity` instead"
                    )));
                }
            }
        }
        (Some(_), Some(_)) | (None, None) => {
            return Err(invalid_params(MOUSE_OVER_PARAMS_HELP));
        }
    };

    let name = world
        .get::<Name>(entity)
        .map(|name| name.as_str().to_string())
        .filter(|name| !name.is_empty());
    Ok(ResolvedTarget { entity, name })
}

/// Finds the first visible mesh with bounds, preferring the root itself and
/// otherwise walking descendants breadth-first.
fn find_mesh_entity(world: &World, root: Entity) -> Option<Entity> {
    let mut queue = VecDeque::from([root]);
    while let Some(entity) = queue.pop_front() {
        let is_visible = world
            .get::<InheritedVisibility>(entity)
            .is_none_or(|visibility| visibility.get());
        if is_visible
            && world.get::<Mesh3d>(entity).is_some()
            && world.get::<Aabb>(entity).is_some()
        {
            return Some(entity);
        }
        if let Some(children) = world.get::<Children>(entity) {
            queue.extend(children.iter());
        }
    }
    None
}

fn project_to_viewport(
    world: &mut World,
    world_position: Vec3,
) -> Result<(Entity, Vec2), bevy::remote::BrpError> {
    let mut query = world.query_filtered::<(Entity, &Camera, &GlobalTransform), With<Camera3d>>();
    let mut cameras = query.iter(world);
    let (camera_entity, camera, camera_transform) = cameras
        .next()
        .ok_or_else(|| invalid_params("no Camera3d found"))?;
    if cameras.next().is_some() {
        return Err(invalid_params(
            "multiple Camera3d components found; expected exactly one",
        ));
    }
    let position = camera
        .world_to_viewport(camera_transform, world_position)
        .map_err(|err| {
            invalid_params(format!(
                "cannot project mesh center {world_position:?} to the viewport: {err}"
            ))
        })?;
    Ok((camera_entity, position))
}

/// Injects the same cursor messages as real winit input and synchronizes the
/// simulated `Window` cursor position.
fn move_simulated_cursor(
    world: &mut World,
    position: Vec2,
) -> Result<Entity, bevy::remote::BrpError> {
    if !position.is_finite() {
        return Err(invalid_params("cursor position is not finite"));
    }

    let mut window_query = world.query_filtered::<Entity, With<PrimaryWindow>>();
    let window_entity = window_query
        .single(world)
        .map_err(|err| invalid_params(format!("expected exactly one primary window: {err}")))?;
    let (window_size, previous_position) = {
        let window = world.get::<Window>(window_entity).ok_or_else(|| {
            invalid_params(format!(
                "primary window entity {} has no Window component",
                window_entity.to_bits()
            ))
        })?;
        (window.size(), window.cursor_position())
    };
    if position.x < 0.0
        || position.y < 0.0
        || position.x >= window_size.x
        || position.y >= window_size.y
    {
        return Err(invalid_params(format!(
            "cursor position {position:?} is outside the primary window (logical size {window_size:?})"
        )));
    }

    let delta = previous_position.map_or(Vec2::ZERO, |previous| position - previous);
    write_input_event(world, MouseMotion { delta });
    write_input_event(
        world,
        CursorMoved {
            window: window_entity,
            position,
            delta: Some(delta),
        },
    );
    let Some(mut window) = world.get_mut::<Window>(window_entity) else {
        return Err(invalid_params(format!(
            "primary window entity {} has no Window component",
            window_entity.to_bits()
        )));
    };
    window.set_cursor_position(Some(position));

    Ok(window_entity)
}

fn write_input_event<T>(world: &mut World, event: T)
where
    T: Clone + Message,
    WindowEvent: From<T>,
{
    // Bevy's winit integration writes both channels: picking reads
    // WindowEvent, while other systems read the concrete message type.
    let _ = world.write_message(WindowEvent::from(event.clone()));
    let _ = world.write_message(event);
}

fn serialize_result<T: Serialize>(result: T) -> BrpResult {
    serde_json::to_value(result).map_err(|err| {
        bevy::remote::BrpError::internal(format!("failed to serialize result: {err}"))
    })
}

struct ResolvedTarget {
    entity: Entity,
    name: Option<String>,
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
